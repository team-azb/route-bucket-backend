use std::convert::TryInto;

use derive_more::From;
use futures::FutureExt;
use getset::Getters;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use route_bucket_domain::model::{
    Coordinate, Distance, Elevation, Operation, Route, RouteGpx, RouteId, RouteInfo, Segment,
    SegmentList,
};
use route_bucket_domain::repository::{
    Connection, ElevationApi, RouteInterpolationApi, RouteRepository,
};
use route_bucket_utils::ApplicationResult;

pub struct RouteUseCase<R, I, E> {
    repository: R,
    interpolation_api: I,
    elevation_api: E,
}

impl<R, I, E> RouteUseCase<R, I, E>
where
    R: RouteRepository,
    I: RouteInterpolationApi,
    E: ElevationApi,
{
    pub fn new(repository: R, interpolation_api: I, elevation_api: E) -> Self {
        Self {
            repository,
            interpolation_api,
            elevation_api,
        }
    }

    pub async fn find(&self, route_id: &RouteId) -> ApplicationResult<RouteGetResponse> {
        let conn = self.repository.get_connection().await?;

        let mut route = self.repository.find(route_id, &conn).await?;
        self.attach_segment_details(route.seg_list_mut()).await?;

        Ok(RouteGetResponse {
            route_info: route.info().clone(),
            waypoints: route.seg_list().gather_waypoints(),
            segments: route.seg_list().clone().into_segments_in_between(),
            elevation_gain: route.calc_elevation_gain(),
            total_distance: route.seg_list().get_total_distance()?,
        })
    }

    pub async fn find_all(&self) -> ApplicationResult<RouteGetAllResponse> {
        let conn = self.repository.get_connection().await?;

        Ok(RouteGetAllResponse {
            route_infos: self.repository.find_all_infos(&conn).await?,
        })
    }

    pub async fn find_gpx(&self, route_id: &RouteId) -> ApplicationResult<RouteGetGpxResponse> {
        let conn = self.repository.get_connection().await?;

        let mut route = self.repository.find(route_id, &conn).await?;
        self.attach_segment_details(route.seg_list_mut()).await?;

        route.try_into()
    }

    pub async fn create(&self, req: &RouteCreateRequest) -> ApplicationResult<RouteCreateResponse> {
        let route_info = RouteInfo::new(RouteId::new(), req.name(), 0);

        let conn = self.repository.get_connection().await?;
        self.repository.insert_info(&route_info, &conn).await?;

        Ok(RouteCreateResponse {
            id: route_info.id().clone(),
        })
    }

    pub async fn rename(
        &self,
        route_id: &RouteId,
        req: &RouteRenameRequest,
    ) -> ApplicationResult<RouteInfo> {
        let conn = self.repository.get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route_info = self.repository.find_info(route_id, conn).await?;
                route_info.rename(req.name());
                self.repository.update_info(&route_info, conn).await?;

                Ok(route_info)
            }
            .boxed()
        })
        .await
    }

    pub async fn add_point(
        &self,
        route_id: &RouteId,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.repository.get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.repository.find(route_id, conn).await?;
                let resp = self
                    .push_op_and_save(
                        &mut route,
                        Operation::new_add(pos, req.coord().clone()),
                        conn,
                    )
                    .await?;

                conn.commit_transaction().await?;

                Ok(resp)
            }
            .boxed()
        })
        .await
    }

    pub async fn remove_point(
        &self,
        route_id: &RouteId,
        pos: usize,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.repository.get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.repository.find(route_id, conn).await?;
                let org_waypoints = route.gather_waypoints();
                let resp = self
                    .push_op_and_save(&mut route, Operation::new_remove(pos, org_waypoints), conn)
                    .await?;

                conn.commit_transaction().await?;

                Ok(resp)
            }
            .boxed()
        })
        .await
    }

    pub async fn move_point(
        &self,
        route_id: &RouteId,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.repository.get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.repository.find(route_id, conn).await?;
                let org_waypoints = route.gather_waypoints();
                let resp = self
                    .push_op_and_save(
                        &mut route,
                        Operation::new_move(pos, req.coord().clone(), org_waypoints),
                        conn,
                    )
                    .await?;

                Ok(resp)
            }
            .boxed()
        })
        .await
    }

    pub async fn clear_route(
        &self,
        route_id: &RouteId,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.repository.get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.repository.find(route_id, conn).await?;
                let org_waypoints = route.gather_waypoints();
                let resp = self
                    .push_op_and_save(&mut route, Operation::new_clear(org_waypoints), conn)
                    .await?;

                Ok(resp)
            }
            .boxed()
        })
        .await
    }

    pub async fn redo_operation(
        &self,
        route_id: &RouteId,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.repository.get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.repository.find(route_id, conn).await?;
                route.redo_operation()?;
                let resp = self.save_edited(&mut route, conn).await?;

                Ok(resp)
            }
            .boxed()
        })
        .await
    }

    pub async fn undo_operation(
        &self,
        route_id: &RouteId,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.repository.get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.repository.find(route_id, conn).await?;
                route.undo_operation()?;
                let resp = self.save_edited(&mut route, conn).await?;

                Ok(resp)
            }
            .boxed()
        })
        .await
    }

    pub async fn delete(&self, route_id: &RouteId) -> ApplicationResult<()> {
        let mut conn = self.repository.get_connection().await?;
        self.repository.delete(route_id, &mut conn).await
    }

    async fn attach_segment_details(&self, seg_list: &mut SegmentList) -> ApplicationResult<()> {
        seg_list.attach_distance_from_start()?;
        seg_list.iter_mut().try_for_each(|seg| {
            seg.iter_mut()
                .filter(|coord| coord.elevation().is_none())
                .try_for_each(|coord| coord.set_elevation(self.elevation_api.get_elevation(coord)?))
        })
    }

    async fn interpolate_and_insert_segment(
        &self,
        route_id: &RouteId,
        pos: u32,
        seg: &mut Segment,
        conn: &R::Connection,
    ) -> ApplicationResult<()> {
        let corrected_start = self
            .interpolation_api
            .correct_coordinate(seg.start())
            .await?;
        let corrected_goal = self
            .interpolation_api
            .correct_coordinate(seg.goal())
            .await?;
        seg.reset_endpoints(Some(corrected_start), Some(corrected_goal));

        self.interpolation_api.interpolate(seg).await?;
        self.repository
            .insert_and_shift_segments(route_id, pos, seg, conn)
            .await?;
        Ok(())
    }

    async fn interpolate_and_update_seg_list(
        &self,
        route_id: &RouteId,
        seg_list: &mut SegmentList,
        conn: &R::Connection,
    ) -> ApplicationResult<()> {
        let range =
            (seg_list.replaced_range().start as u32)..(seg_list.replaced_range().end as u32);
        self.repository
            .delete_and_shift_segments_by_range(route_id, range, conn)
            .await?;

        // 参考：https://www.reddit.com/r/rust/comments/hezhti/help_needed_for_getting_async_and_fnmut_to_work/
        let conn = Mutex::new(conn);

        futures::future::join_all(
            seg_list
                .iter_mut()
                .enumerate()
                .filter(|(_, seg)| seg.is_empty())
                .map(|(i, seg)| {
                    let conn = &conn;
                    async move {
                        self.interpolate_and_insert_segment(
                            route_id,
                            i as u32,
                            seg,
                            &mut *conn.lock().await,
                        )
                        .await
                    }
                }),
        )
        .await
        .into_iter()
        .try_collect()?;

        self.attach_segment_details(seg_list).await
    }

    async fn save_edited(
        &self,
        route: &mut Route,
        conn: &R::Connection,
    ) -> ApplicationResult<RouteOperationResponse> {
        // TODO: posのrangeチェック

        self.repository.update_info(route.info(), conn).await?;
        self.interpolate_and_update_seg_list(
            &route.info().id().clone(),
            route.seg_list_mut(),
            conn,
        )
        .await?;

        // NOTE: どうせここでcloneが必要なので、update_routeの戻り値にSegmentListを指定してもいいかもしれない
        let seg_list = route.seg_list().clone();

        Ok(RouteOperationResponse {
            waypoints: seg_list.gather_waypoints(),
            segments: seg_list.into_segments_in_between(),
            elevation_gain: route.calc_elevation_gain(),
            total_distance: route.seg_list().get_total_distance()?,
        })
    }

    async fn push_op_and_save(
        &self,
        route: &mut Route,
        op: Operation,
        conn: &R::Connection,
    ) -> ApplicationResult<RouteOperationResponse> {
        self.repository
            .insert_and_truncate_operations(
                route.info().id(),
                *route.info().op_num() as u32,
                &op,
                conn,
            )
            .await?;
        route.push_operation(op)?;
        self.save_edited(route, conn).await
    }
}

#[derive(Serialize)]
pub struct RouteGetResponse {
    #[serde(flatten)]
    route_info: RouteInfo,
    waypoints: Vec<Coordinate>,
    segments: Vec<Segment>,
    elevation_gain: Elevation,
    total_distance: Distance,
}

#[derive(Serialize)]
pub struct RouteGetAllResponse {
    #[serde(rename = "routes")]
    route_infos: Vec<RouteInfo>,
}

pub type RouteGetGpxResponse = RouteGpx;

#[derive(From, Getters, Deserialize)]
#[get = "pub"]
pub struct RouteCreateRequest {
    name: String,
}

#[derive(Serialize)]
pub struct RouteCreateResponse {
    pub id: RouteId,
}

#[derive(From, Getters, Deserialize)]
#[get = "pub"]
pub struct NewPointRequest {
    coord: Coordinate,
}

#[derive(Serialize)]
pub struct RouteOperationResponse {
    waypoints: Vec<Coordinate>,
    segments: Vec<Segment>,
    elevation_gain: Elevation,
    total_distance: Distance,
}

#[derive(From, Getters, Deserialize)]
#[get = "pub"]
pub struct RouteRenameRequest {
    name: String,
}
