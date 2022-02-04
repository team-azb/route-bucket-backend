use std::convert::TryInto;

use async_trait::async_trait;
use futures::FutureExt;

pub use requests::*;
pub use responses::*;
use route_bucket_domain::external::{
    CallElevationApi, CallRouteInterpolationApi, CallUserAuthApi, ElevationApi,
    RouteInterpolationApi, UserAuthApi,
};
use route_bucket_domain::model::route::{Operation, Route, RouteId, RouteInfo, RouteSearchQuery};
use route_bucket_domain::repository::{
    CallRouteRepository, Connection, Repository, RouteRepository,
};
use route_bucket_utils::ApplicationResult;

mod requests;
mod responses;

#[async_trait]
pub trait RouteUseCase {
    async fn find(&self, route_id: &RouteId) -> ApplicationResult<RouteGetResponse>;

    async fn find_all(&self) -> ApplicationResult<RouteSearchResponse>;

    async fn search(&self, query: RouteSearchQuery) -> ApplicationResult<RouteSearchResponse>;

    async fn find_gpx(&self, route_id: &RouteId) -> ApplicationResult<RouteGetGpxResponse>;

    async fn create(
        &self,
        user_access_token: &str,
        req: &RouteCreateRequest,
    ) -> ApplicationResult<RouteCreateResponse>;

    async fn rename(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
        req: &RouteRenameRequest,
    ) -> ApplicationResult<RouteInfo>;

    async fn add_point(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse>;

    async fn remove_point(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
        pos: usize,
        req: &RemovePointRequest,
    ) -> ApplicationResult<RouteOperationResponse>;

    async fn move_point(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse>;

    async fn clear_route(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
    ) -> ApplicationResult<RouteOperationResponse>;

    async fn redo_operation(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
    ) -> ApplicationResult<RouteOperationResponse>;

    async fn undo_operation(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
    ) -> ApplicationResult<RouteOperationResponse>;

    async fn delete(&self, route_id: &RouteId, user_access_token: &str) -> ApplicationResult<()>;
}

#[async_trait]
impl<T> RouteUseCase for T
where
    T: CallRouteRepository + CallRouteInterpolationApi + CallElevationApi + CallUserAuthApi + Sync,
{
    async fn find(&self, route_id: &RouteId) -> ApplicationResult<RouteGetResponse> {
        let conn = self.route_repository().get_connection().await?;

        let mut route = self.route_repository().find(route_id, &conn).await?;
        self.elevation_api().attach_elevations(&mut route)?;
        route.calc_route_features_from_seg_list()?;

        route.try_into()
    }

    async fn find_all(&self) -> ApplicationResult<RouteSearchResponse> {
        let conn = self.route_repository().get_connection().await?;

        let route_infos = self
            .route_repository()
            .search_infos(RouteSearchQuery::empty(), &conn)
            .await?;
        let result_num = route_infos.len();

        Ok(RouteSearchResponse {
            route_infos,
            result_num,
        })
    }

    async fn search(&self, query: RouteSearchQuery) -> ApplicationResult<RouteSearchResponse> {
        let conn = self.route_repository().get_connection().await?;

        Ok(RouteSearchResponse {
            route_infos: self
                .route_repository()
                .search_infos(query.clone(), &conn)
                .await?,
            result_num: self.route_repository().count_infos(query, &conn).await?,
        })
    }

    async fn find_gpx(&self, route_id: &RouteId) -> ApplicationResult<RouteGetGpxResponse> {
        let conn = self.route_repository().get_connection().await?;

        let mut route = self.route_repository().find(route_id, &conn).await?;
        self.elevation_api().attach_elevations(&mut route)?;
        route.calc_route_features_from_seg_list()?;

        route.try_into()
    }

    async fn create(
        &self,
        user_access_token: &str,
        req: &RouteCreateRequest,
    ) -> ApplicationResult<RouteCreateResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let owner_id = self.user_auth_api().authenticate(user_access_token).await?;
                let route_info = RouteInfo::new(&req.name, owner_id);

                self.route_repository()
                    .insert_info(&route_info, conn)
                    .await?;

                Ok(RouteCreateResponse {
                    id: route_info.id().clone(),
                })
            }
            .boxed()
        })
        .await
    }

    async fn rename(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
        req: &RouteRenameRequest,
    ) -> ApplicationResult<RouteInfo> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route_info = self.route_repository().find_info(route_id, conn).await?;
                self.user_auth_api()
                    .authorize(route_info.owner_id(), user_access_token)
                    .await?;

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
        user_access_token: &str,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                self.user_auth_api()
                    .authorize(route.info().owner_id(), user_access_token)
                    .await?;

                let op = Operation::new_add(
                    pos,
                    self.route_interpolation_api()
                        .correct_coordinate(&req.coord, req.mode)
                        .await?,
                    route.seg_list(),
                    req.mode,
                )?;
                route.push_operation(op)?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.calc_route_features_from_seg_list()?;

                self.route_repository().update(&route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn remove_point(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
        pos: usize,
        req: &RemovePointRequest,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                self.user_auth_api()
                    .authorize(route.info().owner_id(), user_access_token)
                    .await?;

                let op = Operation::new_remove(pos, route.seg_list(), req.mode)?;
                route.push_operation(op)?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.calc_route_features_from_seg_list()?;

                self.route_repository().update(&route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn move_point(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
        pos: usize,
        req: &NewPointRequest,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                self.user_auth_api()
                    .authorize(route.info().owner_id(), user_access_token)
                    .await?;

                let op = Operation::new_move(
                    pos,
                    self.route_interpolation_api()
                        .correct_coordinate(&req.coord, req.mode)
                        .await?,
                    route.seg_list(),
                    req.mode,
                )?;
                route.push_operation(op)?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.calc_route_features_from_seg_list()?;

                self.route_repository().update(&route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn clear_route(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut info = self.route_repository().find_info(route_id, conn).await?;
                self.user_auth_api()
                    .authorize(info.owner_id(), user_access_token)
                    .await?;

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
        user_access_token: &str,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                self.user_auth_api()
                    .authorize(route.info().owner_id(), user_access_token)
                    .await?;

                route.redo_operation()?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.calc_route_features_from_seg_list()?;

                self.route_repository().update(&route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn undo_operation(
        &self,
        route_id: &RouteId,
        user_access_token: &str,
    ) -> ApplicationResult<RouteOperationResponse> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let mut route = self.route_repository().find(route_id, conn).await?;
                self.user_auth_api()
                    .authorize(route.info().owner_id(), user_access_token)
                    .await?;

                route.undo_operation()?;

                self.route_interpolation_api()
                    .interpolate_empty_segments(&mut route)
                    .await?;
                self.elevation_api().attach_elevations(&mut route)?;
                route.calc_route_features_from_seg_list()?;

                self.route_repository().update(&route, conn).await?;

                route.try_into()
            }
            .boxed()
        })
        .await
    }

    async fn delete(&self, route_id: &RouteId, user_access_token: &str) -> ApplicationResult<()> {
        let conn = self.route_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                let info = self.route_repository().find_info(route_id, conn).await?;
                self.user_auth_api()
                    .authorize(info.owner_id(), user_access_token)
                    .await?;

                self.route_repository().delete(route_id, conn).await
            }
            .boxed()
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::{expect_at_repository, expect_once};
    use route_bucket_domain::{
        external::{MockElevationApi, MockRouteInterpolationApi, MockUserAuthApi},
        model::{
            fixtures::{
                route::{
                    CoordinateFixtures, OperationFixtures, RouteFixtures, RouteGpxFixtures,
                    RouteInfoFixtures, RouteSearchQueryFixtures, SegmentFixtures,
                },
                user::UserIdFixtures,
            },
            route::{Coordinate, DrawingMode, RouteGpx, Segment},
            user::UserId,
        },
        repository::{MockConnection, MockRouteRepository},
    };
    use rstest::rstest;

    use super::*;

    fn route_id() -> RouteId {
        RouteId::from_string("route-id___".into())
    }

    fn tokyo_before_correction() -> Coordinate {
        Coordinate::new(35.68, 139.77).unwrap()
    }

    fn yokohama_to_chiba_before_interpolation(is_undone: bool) -> Route {
        Route::new(
            RouteInfo::empty_route0(if is_undone { 2 } else { 4 }),
            if is_undone {
                Operation::after_add_tokyo_op_list()
            } else {
                Operation::after_remove_tokyo_op_list()
            },
            vec![
                Segment::yokohama_to_chiba(false, None, true, DrawingMode::FollowRoad),
                Segment::chiba(false, None, false, DrawingMode::FollowRoad),
            ]
            .into(),
        )
    }

    fn yokohama_to_chiba_via_tokyo_before_interpolation() -> Route {
        Route::new(
            RouteInfo::empty_route0(3),
            Operation::after_add_tokyo_op_list(),
            vec![
                Segment::yokohama_to_tokyo(false, None, true, DrawingMode::Freehand),
                Segment::tokyo_to_chiba(false, None, true, DrawingMode::Freehand),
                Segment::chiba(false, None, false, DrawingMode::FollowRoad),
            ]
            .into(),
        )
    }

    fn yokohama_to_tokyo_before_interpolation() -> Route {
        Route::yokohama_to_tokyo()
    }

    fn doncic_token() -> String {
        String::from("token.for.doncic")
    }

    #[rstest]
    #[tokio::test]
    async fn can_find() {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_at_route_repository(
            route_id(),
            Route::yokohama_to_chiba_filled(false, false),
        );
        usecase.expect_attach_elevations_at_elevation_api(
            Route::yokohama_to_chiba_filled(false, false),
            Route::yokohama_to_chiba_filled(true, false),
        );

        assert_eq!(
            usecase.find(&route_id()).await,
            Route::yokohama_to_chiba_filled(true, true).try_into()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_find_all() {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_search_infos_at_route_repository(
            RouteSearchQuery::empty(),
            vec![RouteInfo::empty_route0(0)],
        );

        assert_eq!(
            usecase.find_all().await,
            Ok(RouteSearchResponse {
                route_infos: vec![RouteInfo::empty_route0(0)],
                result_num: 1
            })
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_search() {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_search_infos_at_route_repository(
            RouteSearchQuery::search_guest(),
            vec![RouteInfo::empty_route0(0)],
        );
        usecase.expect_count_infos_at_route_repository(RouteSearchQuery::search_guest(), 1);

        assert_eq!(
            usecase.search(RouteSearchQuery::search_guest()).await,
            Ok(RouteSearchResponse {
                route_infos: vec![RouteInfo::empty_route0(0)],
                result_num: 1
            })
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_find_gpx() {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_at_route_repository(
            route_id(),
            Route::yokohama_to_chiba_via_tokyo_filled(false, false),
        );
        usecase.expect_attach_elevations_at_elevation_api(
            Route::yokohama_to_chiba_via_tokyo_filled(false, false),
            Route::yokohama_to_chiba_via_tokyo_filled(true, false),
        );

        assert_eq!(usecase.find_gpx(&route_id()).await, Ok(RouteGpx::route0()));
    }

    #[rstest]
    #[tokio::test]
    async fn can_create() {
        let req = RouteCreateRequest {
            name: "route0".into(),
        };

        let mut usecase = TestRouteUseCase::new();
        usecase.expect_authenticate_at_auth_api(doncic_token(), UserId::doncic());
        usecase.expect_insert_info_at_route_repository(RouteInfo::empty_route0(0));
        // NOTE: unable to check resp since RouteId is auto-generated
        // assert_eq!(usecase.create(&req).await, Ok(expected_resp));
        assert!(matches!(usecase.create(&doncic_token(), &req).await, Ok(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn can_rename() {
        let req = RouteRenameRequest {
            name: "route1".into(),
        };

        let mut usecase = TestRouteUseCase::new();
        usecase.expect_authorize_at_auth_api(UserId::doncic(), doncic_token());
        usecase.expect_find_info_at_route_repository(route_id(), RouteInfo::empty_route0(0));
        usecase.expect_update_info_at_route_repository(RouteInfo::empty_route1(0));

        assert_eq!(
            usecase.rename(&route_id(), &doncic_token(), &req).await,
            Ok(RouteInfo::empty_route1(0))
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_add_point() {
        let req = NewPointRequest {
            mode: DrawingMode::Freehand,
            coord: tokyo_before_correction(),
        };

        let mut usecase = TestRouteUseCase::new();
        usecase.expect_authorize_at_auth_api(UserId::doncic(), doncic_token());
        usecase.expect_find_at_route_repository(
            route_id(),
            Route::yokohama_to_chiba_filled(false, false),
        );
        usecase.expect_correct_coordinate_at_interpolation_api(
            tokyo_before_correction(),
            DrawingMode::Freehand,
            Coordinate::tokyo(false, None),
        );
        usecase.expect_interpolate_empty_segments_at_interpolation_api(
            yokohama_to_chiba_via_tokyo_before_interpolation(),
            Route::yokohama_to_chiba_via_tokyo_filled(false, false),
        );
        usecase.expect_attach_elevations_at_elevation_api(
            Route::yokohama_to_chiba_via_tokyo_filled(false, false),
            Route::yokohama_to_chiba_via_tokyo_filled(true, false),
        );
        usecase.expect_update_at_route_repository(Route::yokohama_to_chiba_via_tokyo_filled(
            true, true,
        ));

        assert_eq!(
            usecase
                .add_point(&route_id(), &doncic_token(), 1, &req)
                .await,
            Route::yokohama_to_chiba_via_tokyo_filled(true, true).try_into()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_remove_point() {
        let req = RemovePointRequest {
            mode: DrawingMode::FollowRoad,
        };

        let mut usecase = TestRouteUseCase::new();
        usecase.expect_authorize_at_auth_api(UserId::doncic(), doncic_token());
        usecase.expect_find_at_route_repository(
            route_id(),
            Route::yokohama_to_chiba_via_tokyo_filled(false, false),
        );
        usecase.expect_interpolate_empty_segments_at_interpolation_api(
            yokohama_to_chiba_before_interpolation(false),
            Route::yokohama_to_chiba_filled(false, false),
        );
        usecase.expect_attach_elevations_at_elevation_api(
            Route::yokohama_to_chiba_filled(false, false),
            Route::yokohama_to_chiba_filled(true, false),
        );
        usecase.expect_update_at_route_repository(Route::yokohama_to_chiba_filled(true, true));

        assert_eq!(
            usecase
                .remove_point(&route_id(), &doncic_token(), 1, &req)
                .await,
            Route::yokohama_to_chiba_filled(true, true).try_into()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_move_point() {
        let req = NewPointRequest {
            mode: DrawingMode::Freehand,
            coord: tokyo_before_correction(),
        };

        let mut usecase = TestRouteUseCase::new();
        usecase.expect_authorize_at_auth_api(UserId::doncic(), doncic_token());
        usecase.expect_find_at_route_repository(
            route_id(),
            Route::yokohama_to_chiba_filled(false, false),
        );
        usecase.expect_correct_coordinate_at_interpolation_api(
            tokyo_before_correction(),
            DrawingMode::Freehand,
            Coordinate::tokyo(false, None),
        );
        usecase.expect_interpolate_empty_segments_at_interpolation_api(
            yokohama_to_tokyo_before_interpolation(),
            Route::yokohama_to_tokyo_filled(false, false),
        );
        usecase.expect_attach_elevations_at_elevation_api(
            Route::yokohama_to_tokyo_filled(false, false),
            Route::yokohama_to_tokyo_filled(true, false),
        );
        usecase.expect_update_at_route_repository(Route::yokohama_to_tokyo_filled(true, true));

        assert_eq!(
            usecase
                .move_point(&route_id(), &doncic_token(), 1, &req)
                .await,
            Route::yokohama_to_tokyo_filled(true, true).try_into()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_clear_route() {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_authorize_at_auth_api(UserId::doncic(), doncic_token());
        usecase.expect_find_info_at_route_repository(route_id(), RouteInfo::empty_route0(3));
        usecase.expect_update_at_route_repository(Route::empty());

        assert_eq!(
            usecase.clear_route(&route_id(), &doncic_token()).await,
            Route::empty().try_into()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_redo_operation() {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_authorize_at_auth_api(UserId::doncic(), doncic_token());
        usecase.expect_find_at_route_repository(
            route_id(),
            Route::yokohama_to_chiba_filled(false, false),
        );
        usecase.expect_interpolate_empty_segments_at_interpolation_api(
            yokohama_to_chiba_via_tokyo_before_interpolation(),
            Route::yokohama_to_chiba_via_tokyo_filled(false, false),
        );
        usecase.expect_attach_elevations_at_elevation_api(
            Route::yokohama_to_chiba_via_tokyo_filled(false, false),
            Route::yokohama_to_chiba_via_tokyo_filled(true, false),
        );
        usecase.expect_update_at_route_repository(Route::yokohama_to_chiba_via_tokyo_filled(
            true, true,
        ));

        assert_eq!(
            usecase.redo_operation(&route_id(), &doncic_token()).await,
            Route::yokohama_to_chiba_via_tokyo_filled(true, true).try_into()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_undo_operation() {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_authorize_at_auth_api(UserId::doncic(), doncic_token());
        usecase.expect_find_at_route_repository(
            route_id(),
            Route::yokohama_to_chiba_via_tokyo_filled(false, false),
        );
        usecase.expect_interpolate_empty_segments_at_interpolation_api(
            yokohama_to_chiba_before_interpolation(true),
            Route::yokohama_to_chiba_filled(false, false),
        );
        usecase.expect_attach_elevations_at_elevation_api(
            Route::yokohama_to_chiba_filled(false, false),
            Route::yokohama_to_chiba_filled(true, false),
        );
        usecase.expect_update_at_route_repository(Route::yokohama_to_chiba_filled(true, true));

        assert_eq!(
            usecase.undo_operation(&route_id(), &doncic_token()).await,
            Route::yokohama_to_chiba_filled(true, true).try_into()
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_delete() {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_info_at_route_repository(route_id(), RouteInfo::empty_route0(0));
        usecase.expect_authorize_at_auth_api(UserId::doncic(), doncic_token());
        usecase.expect_delete_at_route_repository(route_id());
        assert_eq!(usecase.delete(&route_id(), &doncic_token()).await, Ok(()));
    }

    struct TestRouteUseCase {
        repository: MockRouteRepository,
        interpolation_api: MockRouteInterpolationApi,
        elevation_api: MockElevationApi,
        auth_api: MockUserAuthApi,
    }

    // setup methods for mocking
    impl TestRouteUseCase {
        fn new() -> Self {
            let mut usecase = TestRouteUseCase {
                repository: MockRouteRepository::new(),
                interpolation_api: MockRouteInterpolationApi::new(),
                elevation_api: MockElevationApi::new(),
                auth_api: MockUserAuthApi::new(),
            };
            expect_at_repository!(usecase, get_connection, MockConnection {});

            usecase
        }

        fn expect_find_at_route_repository(&mut self, param_id: RouteId, return_route: Route) {
            expect_at_repository!(self, find, param_id, return_route);
        }

        fn expect_find_info_at_route_repository(
            &mut self,
            param_id: RouteId,
            return_info: RouteInfo,
        ) {
            expect_at_repository!(self, find_info, param_id, return_info);
        }

        fn expect_search_infos_at_route_repository(
            &mut self,
            query: RouteSearchQuery,
            return_infos: Vec<RouteInfo>,
        ) {
            expect_at_repository!(self, search_infos, query, return_infos);
        }

        fn expect_count_infos_at_route_repository(
            &mut self,
            query: RouteSearchQuery,
            return_count: usize,
        ) {
            expect_at_repository!(self, count_infos, query, return_count);
        }

        fn expect_insert_info_at_route_repository(&mut self, param_info: RouteInfo) {
            expect_at_repository!(self, insert_info, param_info, ());
        }

        fn expect_update_at_route_repository(&mut self, param_route: Route) {
            expect_at_repository!(self, update, param_route, ());
        }

        fn expect_update_info_at_route_repository(&mut self, param_info: RouteInfo) {
            expect_at_repository!(self, update_info, param_info, ());
        }

        fn expect_delete_at_route_repository(&mut self, param_id: RouteId) {
            expect_at_repository!(self, delete, param_id, ());
        }

        fn expect_correct_coordinate_at_interpolation_api(
            &mut self,
            param_coord: Coordinate,
            param_mode: DrawingMode,
            return_coord: Coordinate,
        ) {
            expect_once!(
                self.interpolation_api,
                correct_coordinate,
                param_coord,
                param_mode,
                return_coord
            );
        }

        fn expect_interpolate_empty_segments_at_interpolation_api(
            &mut self,
            before_route: Route,
            after_route: Route,
        ) {
            expect_once!(
                self.interpolation_api,
                interpolate_empty_segments,
                before_route => after_route
            );
        }

        fn expect_attach_elevations_at_elevation_api(
            &mut self,
            before_route: Route,
            after_route: Route,
        ) {
            expect_once!(
                self.elevation_api,
                attach_elevations,
                before_route => after_route
            );
        }

        fn expect_authenticate_at_auth_api(&mut self, param_token: String, return_id: UserId) {
            expect_once!(self.auth_api, authenticate, param_token, return_id);
        }

        fn expect_authorize_at_auth_api(&mut self, param_id: UserId, param_token: String) {
            expect_once!(self.auth_api, authorize, param_id, param_token, ());
        }
    }

    // impls to enable trait RouteUseCase
    impl CallRouteRepository for TestRouteUseCase {
        type RouteRepository = MockRouteRepository;

        fn route_repository(&self) -> &Self::RouteRepository {
            &self.repository
        }
    }

    impl CallRouteInterpolationApi for TestRouteUseCase {
        type RouteInterpolationApi = MockRouteInterpolationApi;

        fn route_interpolation_api(&self) -> &Self::RouteInterpolationApi {
            &self.interpolation_api
        }
    }

    impl CallElevationApi for TestRouteUseCase {
        type ElevationApi = MockElevationApi;

        fn elevation_api(&self) -> &Self::ElevationApi {
            &self.elevation_api
        }
    }

    impl CallUserAuthApi for TestRouteUseCase {
        type UserAuthApi = MockUserAuthApi;

        fn user_auth_api(&self) -> &Self::UserAuthApi {
            &self.auth_api
        }
    }
}
