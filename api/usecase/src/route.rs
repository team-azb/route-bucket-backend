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

#[cfg(test)]
mod tests {
    use route_bucket_domain::{
        external::{MockElevationApi, MockRouteInterpolationApi},
        model::{
            fixtures::{
                CoordinateFixtures, OperationFixtures, RouteFixtures, RouteGpxFixtures,
                RouteInfoFixtures, SegmentFixtures, SegmentListFixture,
            },
            Coordinate, Distance, Elevation, RouteGpx, Segment, SegmentList,
        },
        repository::{MockConnection, MockRouteRepository},
    };
    use rstest::{fixture, rstest};

    use super::*;

    #[fixture]
    fn base_route_id() -> RouteId {
        RouteId::from_string("base_route_".into())
    }

    #[fixture]
    fn added_route_id() -> RouteId {
        RouteId::from_string("added_route".into())
    }

    #[fixture]
    fn route_get_resp() -> RouteGetResponse {
        RouteGetResponse {
            route_info: RouteInfo::route0(2),
            waypoints: Coordinate::yokohama_to_chiba_coords(false, None),
            segments: SegmentList::yokohama_to_chiba(true, true, false).into_segments_in_between(),
            elevation_gain: 10.try_into().unwrap(),
            total_distance: 46779.709825324135.try_into().unwrap(),
        }
    }

    #[fixture]
    fn route_get_all_resp() -> RouteGetAllResponse {
        RouteGetAllResponse {
            route_infos: vec![RouteInfo::route0(0)],
        }
    }

    #[fixture]
    fn route_get_gpx_resp() -> RouteGetGpxResponse {
        RouteGpx::route0()
    }

    #[fixture]
    fn route_create_req() -> RouteCreateRequest {
        RouteCreateRequest {
            name: "route0".into(),
        }
    }

    #[fixture]
    fn route_rename_req() -> RouteRenameRequest {
        RouteRenameRequest {
            name: "route1".into(),
        }
    }

    #[fixture]
    fn route_rename_resp() -> RouteInfo {
        RouteInfo::new(RouteId::new(), &"route1".into(), 2)
    }

    #[fixture]
    fn tokyo_before_correction() -> Coordinate {
        Coordinate::new(35.68, 139.77).unwrap()
    }

    #[fixture]
    fn new_point_req(tokyo_before_correction: Coordinate) -> NewPointRequest {
        NewPointRequest {
            coord: tokyo_before_correction,
        }
    }

    #[fixture]
    fn add_point_resp() -> RouteOperationResponse {
        RouteOperationResponse {
            waypoints: Coordinate::yokohama_to_chiba_via_tokyo_coords(false, None),
            segments: SegmentList::yokohama_to_chiba_via_tokyo(true, true, false)
                .into_segments_in_between(),
            elevation_gain: 10.try_into().unwrap(),
            total_distance: 58759.973932514884.try_into().unwrap(),
        }
    }

    #[fixture]
    fn remove_point_resp() -> RouteOperationResponse {
        RouteOperationResponse {
            waypoints: Coordinate::yokohama_to_chiba_coords(false, None),
            segments: SegmentList::yokohama_to_chiba(true, true, false).into_segments_in_between(),
            elevation_gain: 10.try_into().unwrap(),
            total_distance: 46779.709825324135.try_into().unwrap(),
        }
    }

    #[fixture]
    fn move_point_resp() -> RouteOperationResponse {
        RouteOperationResponse {
            waypoints: Coordinate::yokohama_to_tokyo_coords(false, None),
            segments: SegmentList::yokohama_to_tokyo(true, true, false).into_segments_in_between(),
            elevation_gain: 3.try_into().unwrap(),
            total_distance: 26936.42633640023.try_into().unwrap(),
        }
    }

    #[fixture]
    fn clear_route_resp() -> RouteOperationResponse {
        RouteOperationResponse {
            waypoints: Vec::new(),
            segments: Vec::new(),
            elevation_gain: Elevation::zero(),
            total_distance: Distance::zero(),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn can_find(
        #[from(base_route_id)] route_id: RouteId,
        #[from(route_get_resp)] expected_resp: RouteGetResponse,
    ) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_at_route_repository();
        usecase.expect_attach_elevations_at_elevation_api();
        assert_eq!(usecase.find(&route_id).await, Ok(expected_resp));
    }

    #[rstest]
    #[tokio::test]
    async fn can_find_all(#[from(route_get_all_resp)] expected_resp: RouteGetAllResponse) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_all_at_route_repository();
        assert_eq!(usecase.find_all().await, Ok(expected_resp));
    }

    #[rstest]
    #[tokio::test]
    async fn can_find_gpx(
        #[from(added_route_id)] route_id: RouteId,
        #[from(route_get_gpx_resp)] expected_resp: RouteGetGpxResponse,
    ) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_at_route_repository();
        usecase.expect_attach_elevations_at_elevation_api();
        assert_eq!(usecase.find_gpx(&route_id).await, Ok(expected_resp));
    }

    #[rstest]
    #[tokio::test]
    async fn can_create(#[from(route_create_req)] req: RouteCreateRequest) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_insert_info_at_route_repository();
        assert!(matches!(usecase.create(&req).await, Ok(_)));
        // NOTE: unable to check resp since RouteId is auto-generated
        // assert_eq!(usecase.create(&req).await, Ok(expected_resp));
    }

    #[rstest]
    #[tokio::test]
    async fn can_rename(
        #[from(base_route_id)] route_id: RouteId,
        #[from(route_rename_req)] req: RouteRenameRequest,
        #[from(route_rename_resp)] expected_resp: RouteInfo,
    ) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_info_at_route_repository();
        usecase.expect_update_info_at_route_repository();
        assert_eq!(usecase.rename(&route_id, &req).await, Ok(expected_resp));
    }

    #[rstest]
    #[tokio::test]
    async fn can_add_point(
        #[from(base_route_id)] route_id: RouteId,
        #[from(new_point_req)] req: NewPointRequest,
        #[from(add_point_resp)] expected_resp: RouteOperationResponse,
    ) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_at_route_repository();
        usecase.expect_correct_coordinate_at_interpolation_api();
        usecase.expect_interpolate_empty_segments_at_interpolation_api();
        usecase.expect_attach_elevations_at_elevation_api();
        usecase.expect_update_at_route_repository();
        assert_eq!(
            usecase.add_point(&route_id, 1, &req).await,
            Ok(expected_resp)
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_remove_point(
        #[from(added_route_id)] route_id: RouteId,
        #[from(remove_point_resp)] expected_resp: RouteOperationResponse,
    ) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_at_route_repository();
        usecase.expect_interpolate_empty_segments_at_interpolation_api();
        usecase.expect_attach_elevations_at_elevation_api();
        usecase.expect_update_at_route_repository();
        assert_eq!(usecase.remove_point(&route_id, 1).await, Ok(expected_resp));
    }

    #[rstest]
    #[tokio::test]
    async fn can_move_point(
        #[from(base_route_id)] route_id: RouteId,
        #[from(new_point_req)] req: NewPointRequest,
        #[from(move_point_resp)] expected_resp: RouteOperationResponse,
    ) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_at_route_repository();
        usecase.expect_correct_coordinate_at_interpolation_api();
        usecase.expect_interpolate_empty_segments_at_interpolation_api();
        usecase.expect_attach_elevations_at_elevation_api();
        usecase.expect_update_at_route_repository();
        assert_eq!(
            usecase.move_point(&route_id, 1, &req).await,
            Ok(expected_resp)
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_clear_route(
        #[from(base_route_id)] route_id: RouteId,
        #[from(clear_route_resp)] expected_resp: RouteOperationResponse,
    ) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_info_at_route_repository();
        usecase.expect_update_at_route_repository();
        assert_eq!(usecase.clear_route(&route_id).await, Ok(expected_resp));
    }

    #[rstest]
    #[tokio::test]
    async fn can_redo_operation(
        #[from(base_route_id)] route_id: RouteId,
        #[from(add_point_resp)] expected_resp: RouteOperationResponse,
    ) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_at_route_repository();
        usecase.expect_interpolate_empty_segments_at_interpolation_api();
        usecase.expect_attach_elevations_at_elevation_api();
        usecase.expect_update_at_route_repository();
        assert_eq!(usecase.redo_operation(&route_id).await, Ok(expected_resp));
    }

    #[rstest]
    #[tokio::test]
    async fn can_undo_operation(
        #[from(added_route_id)] route_id: RouteId,
        #[from(remove_point_resp)] expected_resp: RouteOperationResponse,
    ) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_find_at_route_repository();
        usecase.expect_interpolate_empty_segments_at_interpolation_api();
        usecase.expect_attach_elevations_at_elevation_api();
        usecase.expect_update_at_route_repository();
        assert_eq!(usecase.undo_operation(&route_id).await, Ok(expected_resp));
    }

    #[rstest]
    #[tokio::test]
    async fn can_delete(#[from(base_route_id)] route_id: RouteId) {
        let mut usecase = TestRouteUseCase::new();
        usecase.expect_delete_at_route_repository();
        assert_eq!(usecase.delete(&route_id).await, Ok(()));
    }

    struct TestRouteUseCase {
        repository: MockRouteRepository,
        interpolation_api: MockRouteInterpolationApi,
        elevation_api: MockElevationApi,
    }

    // setup methods for mocking
    impl TestRouteUseCase {
        fn new() -> Self {
            let mut usecase = TestRouteUseCase {
                repository: MockRouteRepository::new(),
                interpolation_api: MockRouteInterpolationApi::new(),
                elevation_api: MockElevationApi::new(),
            };

            usecase
                .repository
                .expect_get_connection()
                .times(..=1)
                .return_const(Ok(MockConnection {}));

            usecase
        }

        fn base_route(set_ele: bool, set_dist: bool) -> Route {
            Route::new(
                RouteInfo::route0(2),
                Operation::after_add_tokyo_op_list(),
                SegmentList::yokohama_to_chiba(set_ele, set_dist, false),
            )
        }

        fn base_route_after_remove() -> Route {
            Route::new(
                RouteInfo::route0(4),
                Operation::after_remove_tokyo_op_list(),
                vec![
                    Segment::yokohama_to_chiba(false, None, true),
                    Segment::chiba(false, None, false),
                ]
                .into(),
            )
        }

        fn base_route_after_undo() -> Route {
            Route::new(
                RouteInfo::route0(2),
                Operation::after_add_tokyo_op_list(),
                vec![
                    Segment::yokohama_to_chiba(false, None, true),
                    Segment::chiba(false, None, false),
                ]
                .into(),
            )
        }

        fn added_route(set_ele: bool, set_dist: bool) -> Route {
            Route::new(
                RouteInfo::route0(3),
                Operation::after_add_tokyo_op_list(),
                SegmentList::yokohama_to_chiba_via_tokyo(set_ele, set_dist, false),
            )
        }

        fn added_route_before_interpolation() -> Route {
            Route::new(
                RouteInfo::route0(3),
                Operation::after_add_tokyo_op_list(),
                vec![
                    Segment::yokohama_to_tokyo(false, None, true),
                    Segment::tokyo_to_chiba(false, None, true),
                    Segment::chiba(false, None, false),
                ]
                .into(),
            )
        }

        fn moved_route(set_ele: bool, set_dist: bool) -> Route {
            Route::new(
                RouteInfo::route0(3),
                Operation::after_move_chiba_op_list(),
                SegmentList::yokohama_to_tokyo(set_ele, set_dist, false),
            )
        }

        fn moved_route_before_interpolation() -> Route {
            Route::yokohama_to_tokyo()
        }

        fn expect_find_at_route_repository(&mut self) {
            self.repository
                .expect_find()
                .once()
                // panicking on match failure instead of adding `Expectation::with`
                // so that we can see what's in `id`
                .returning(move |id, _| {
                    Ok(match id {
                        id if *id == base_route_id() => Self::base_route(false, false),
                        id if *id == added_route_id() => Self::added_route(false, false),
                        id => panic!("unexpected id: {:#?}", id),
                    })
                });
        }

        fn expect_find_info_at_route_repository(&mut self) {
            self.repository
                .expect_find_info()
                .once()
                .returning(move |id, _| {
                    Ok(match id {
                        id if *id == base_route_id() => RouteInfo::route0(2),
                        id if *id == added_route_id() => RouteInfo::route0(3),
                        id => panic!("unexpected id: {:#?}", id),
                    })
                });
        }

        fn expect_find_all_at_route_repository(&mut self) {
            self.repository
                .expect_find_all_infos()
                .once()
                .return_const(Ok(vec![RouteInfo::route0(0)]));
        }

        fn expect_insert_info_at_route_repository(&mut self) {
            self.repository
                .expect_insert_info()
                .once()
                .returning(move |info, _| {
                    Ok(match info {
                        info if *info == RouteInfo::route0(0) => (),
                        info => panic!("unexpected info: {:#?}", info),
                    })
                });
        }

        fn expect_update_at_route_repository(&mut self) {
            self.repository
                .expect_update()
                .once()
                .returning(move |route, _| {
                    Ok(match route {
                        route if *route == Self::base_route(true, true) => (),
                        route if *route == Self::added_route(true, true) => (),
                        route if *route == Self::moved_route(true, true) => (),
                        route if *route == Route::empty() => (),
                        route => panic!("unexpected route: {:#?}", route),
                    })
                });
        }

        fn expect_update_info_at_route_repository(&mut self) {
            self.repository
                .expect_update_info()
                .once()
                .returning(move |info, _| {
                    Ok(match info {
                        info if *info == route_rename_resp() => (),
                        info => panic!("unexpected info: {:#?}", info),
                    })
                });
        }

        fn expect_delete_at_route_repository(&mut self) {
            self.repository
                .expect_delete()
                .once()
                .returning(move |id, _| {
                    Ok(match id {
                        id if *id == base_route_id() => (),
                        id => panic!("unexpected id: {:#?}", id),
                    })
                });
        }

        fn expect_correct_coordinate_at_interpolation_api(&mut self) {
            self.interpolation_api
                .expect_correct_coordinate()
                .once()
                .returning(move |coord| {
                    Ok(match coord {
                        coord if *coord == tokyo_before_correction() => {
                            Coordinate::tokyo(false, None)
                        }
                        coord => panic!("unexpected coord: {:#?}", coord),
                    })
                });
        }

        fn expect_interpolate_empty_segments_at_interpolation_api(&mut self) {
            self.interpolation_api
                .expect_interpolate_empty_segments()
                .once()
                .returning(move |route| {
                    Ok(match route {
                        route
                            if *route == Self::base_route_after_remove()
                                || *route == Self::base_route_after_undo() =>
                        {
                            *route = Self::base_route(false, false);
                        }
                        route if *route == Self::added_route_before_interpolation() => {
                            *route = Self::added_route(false, false);
                        }
                        route if *route == Self::moved_route_before_interpolation() => {
                            *route = Self::moved_route(false, false);
                        }
                        route => panic!("unexpected route: {:#?}", route),
                    })
                });
        }

        fn expect_attach_elevations_at_elevation_api(&mut self) {
            self.elevation_api
                .expect_attach_elevations()
                .once()
                .returning(move |route| {
                    Ok(match route {
                        route if *route == Self::base_route(false, false) => {
                            *route = Self::base_route(true, false);
                        }
                        route if *route == Self::base_route(false, true) => {
                            *route = Self::base_route(true, true);
                        }
                        route if *route == Self::added_route(false, false) => {
                            *route = Self::added_route(true, false);
                        }
                        route if *route == Self::added_route(false, true) => {
                            *route = Self::added_route(true, true);
                        }
                        route if *route == Self::moved_route(false, false) => {
                            *route = Self::moved_route(true, false);
                        }
                        route if *route == Self::moved_route(false, true) => {
                            *route = Self::moved_route(true, true);
                        }
                        route => panic!("unexpected route: {:#?}", route),
                    })
                });
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
}
