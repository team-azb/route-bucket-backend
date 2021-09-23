use std::convert::TryInto;

use async_trait::async_trait;
use futures::FutureExt;

pub use requests::*;
pub use responses::*;
use route_bucket_domain::external::{
    CallElevationApi, CallRouteInterpolationApi, ElevationApi, RouteInterpolationApi,
};
use route_bucket_domain::model::{Operation, Route, RouteId, RouteInfo};
use route_bucket_domain::repository::{
    CallRouteRepository, Connection, Repository, RouteRepository,
};
use route_bucket_utils::ApplicationResult;

mod requests;
mod responses;

#[async_trait]
pub trait RouteUseCase {
    async fn find(&self, route_id: &RouteId) -> ApplicationResult<RouteGetResponse>;

    async fn find_all(&self) -> ApplicationResult<RouteGetAllResponse>;

    async fn find_gpx(&self, route_id: &RouteId) -> ApplicationResult<RouteGetGpxResponse>;

    async fn create(&self, req: &RouteCreateRequest) -> ApplicationResult<RouteCreateResponse>;

    async fn rename(
        &self,
        route_id: &RouteId,
        req: &RouteRenameRequest,
    ) -> ApplicationResult<RouteInfo>;

    async fn add_point(
        &self,
        route_id: &RouteId,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse>;

    async fn remove_point(
        &self,
        route_id: &RouteId,
        pos: usize,
    ) -> ApplicationResult<RouteOperationResponse>;

    async fn move_point(
        &self,
        route_id: &RouteId,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse>;

    async fn clear_route(&self, route_id: &RouteId) -> ApplicationResult<RouteOperationResponse>;

    async fn redo_operation(&self, route_id: &RouteId)
        -> ApplicationResult<RouteOperationResponse>;

    async fn undo_operation(&self, route_id: &RouteId)
        -> ApplicationResult<RouteOperationResponse>;

    async fn delete(&self, route_id: &RouteId) -> ApplicationResult<()>;
}

#[async_trait]
impl<T> RouteUseCase for T
where
    T: CallRouteRepository + CallRouteInterpolationApi + CallElevationApi + Sync,
{
    async fn find(&self, route_id: &RouteId) -> ApplicationResult<RouteGetResponse> {
        let conn = self.route_repository().get_connection().await?;

        let mut route = self.route_repository().find(route_id, &conn).await?;
        route.attach_distance_from_start()?;
        self.elevation_api().attach_elevations(&mut route)?;

        route.try_into()
    }

    async fn find_all(&self) -> ApplicationResult<RouteGetAllResponse> {
        let conn = self.route_repository().get_connection().await?;

        Ok(RouteGetAllResponse {
            route_infos: self.route_repository().find_all_infos(&conn).await?,
        })
    }

    async fn find_gpx(&self, route_id: &RouteId) -> ApplicationResult<RouteGetGpxResponse> {
        let conn = self.route_repository().get_connection().await?;

        let mut route = self.route_repository().find(route_id, &conn).await?;
        route.attach_distance_from_start()?;
        self.elevation_api().attach_elevations(&mut route)?;

        route.try_into()
    }

    async fn create(&self, req: &RouteCreateRequest) -> ApplicationResult<RouteCreateResponse> {
        let route_info = RouteInfo::new(RouteId::new(), &req.name, 0);

        let conn = self.route_repository().get_connection().await?;
        self.route_repository()
            .insert_info(&route_info, &conn)
            .await?;

        Ok(RouteCreateResponse {
            id: route_info.id().clone(),
        })
    }

    async fn rename(
        &self,
        route_id: &RouteId,
        req: &RouteRenameRequest,
    ) -> ApplicationResult<RouteInfo> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route_info = self.route_repository().find_info(route_id, conn).await?;
                route_info.rename(&req.name);
                self.route_repository()
                    .update_info(&route_info, conn)
                    .await?;

                Ok(route_info)
            }
            .boxed()
        })
        .await
    }

    async fn add_point(
        &self,
        route_id: &RouteId,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                let op = Operation::new_add(
                    pos,
                    self.route_interpolation_api()
                        .correct_coordinate(&req.coord)
                        .await?,
                );
                route.push_operation(op)?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.attach_distance_from_start()?;

                self.route_repository().update(&mut route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn remove_point(
        &self,
        route_id: &RouteId,
        pos: usize,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                let op = Operation::new_remove(pos, route.gather_waypoints());
                route.push_operation(op)?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.attach_distance_from_start()?;

                self.route_repository().update(&mut route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn move_point(
        &self,
        route_id: &RouteId,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                let op = Operation::new_move(
                    pos,
                    self.route_interpolation_api()
                        .correct_coordinate(&req.coord)
                        .await?,
                    route.gather_waypoints(),
                );
                route.push_operation(op)?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.attach_distance_from_start()?;

                self.route_repository().update(&mut route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn clear_route(&self, route_id: &RouteId) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut info = self.route_repository().find_info(route_id, conn).await?;
                info.clear_route();
                let cleared_route = Route::new(info, vec![], vec![].into());
                self.route_repository().update(&cleared_route, conn).await?;

                // TODO: ここは正直無駄なので、APIを変更するべき？
                cleared_route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn redo_operation(
        &self,
        route_id: &RouteId,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                route.redo_operation()?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.attach_distance_from_start()?;

                self.route_repository().update(&mut route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn undo_operation(
        &self,
        route_id: &RouteId,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                route.undo_operation()?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.attach_distance_from_start()?;

                self.route_repository().update(&mut route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn delete(&self, route_id: &RouteId) -> ApplicationResult<()> {
        let mut conn = self.route_repository().get_connection().await?;
        self.route_repository().delete(route_id, &mut conn).await
    }
}
