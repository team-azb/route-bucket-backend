use itertools::Itertools;

use crate::domain::model::coordinate::Coordinate;
use crate::domain::model::route::{Route, RouteInfo};
use crate::domain::model::segment::SegmentList;
use crate::domain::model::types::RouteId;
use crate::domain::repository::{
    ElevationApi, OperationRepository, RouteInterpolationApi, RouteRepository, SegmentRepository,
};
use crate::utils::error::ApplicationResult;

pub struct RouteService<R, O, S, I, E> {
    route_repository: R,
    operation_repository: O,
    segment_repository: S,
    interpolation_api: I,
    elevation_api: E,
}

// TODO: メソッドが多すぎるので、複数のtraitに分ける
impl<R, O, S, I, E> RouteService<R, O, S, I, E>
where
    R: RouteRepository,
    O: OperationRepository,
    S: SegmentRepository,
    I: RouteInterpolationApi,
    E: ElevationApi,
{
    pub fn new(
        route_repository: R,
        operation_repository: O,
        segment_repository: S,
        interpolation_api: I,
        elevation_api: E,
    ) -> Self {
        Self {
            route_repository,
            operation_repository,
            segment_repository,
            interpolation_api,
            elevation_api,
        }
    }

    pub fn find_info(&self, route_id: &RouteId) -> ApplicationResult<RouteInfo> {
        self.route_repository.find(route_id)
    }

    pub fn find_all_infos(&self) -> ApplicationResult<Vec<RouteInfo>> {
        self.route_repository.find_all()
    }

    pub fn find_route(&self, route_id: &RouteId) -> ApplicationResult<Route> {
        let route_info = self.find_info(route_id)?;
        let op_list = self.operation_repository.find_by_route_id(route_id)?;
        let seg_list = self.find_segment_list(route_id)?;

        Ok(Route::new(route_info, op_list, seg_list))
    }

    pub fn find_segment_list(&self, route_id: &RouteId) -> ApplicationResult<SegmentList> {
        let mut seg_list = self.segment_repository.find_by_route_id(route_id)?;
        self.attach_elevation(&mut seg_list)?;
        Ok(seg_list)
    }

    pub fn update_info(&self, info: &RouteInfo) -> ApplicationResult<()> {
        self.route_repository.update(info)
    }

    pub fn update_route(&self, route: &mut Route) -> ApplicationResult<()> {
        self.update_info(route.info())?;
        self.operation_repository
            .update_by_route_id(route.info().id(), route.op_list())?;

        self.update_segment_list(&route.info().id().clone(), route.seg_list_mut())?;
        self.attach_elevation(route.seg_list_mut())?;
        route.seg_list_mut().attach_distance_from_start()
    }

    fn update_segment_list(
        &self,
        route_id: &RouteId,
        seg_list: &mut SegmentList,
    ) -> ApplicationResult<()> {
        let range =
            (seg_list.replaced_range().start as u32)..(seg_list.replaced_range().end as u32);
        self.segment_repository
            .delete_by_route_id_and_range(route_id, range)?;

        seg_list
            .iter_mut()
            .enumerate()
            .filter(|(_, seg)| seg.is_empty())
            .try_for_each(|(i, seg)| {
                let corrected_start = self.interpolation_api.correct_coordinate(seg.start())?;
                let corrected_goal = self.interpolation_api.correct_coordinate(seg.goal())?;
                seg.reset_endpoints(Some(corrected_start), Some(corrected_goal));

                self.interpolation_api.interpolate(seg)?;
                self.segment_repository.insert(route_id, i as u32, seg)?;
                Ok(())
            })
    }

    pub fn register_route(&self, route_info: &RouteInfo) -> ApplicationResult<()> {
        self.route_repository.register(route_info)
    }

    pub fn delete_route(&self, route_id: &RouteId) -> ApplicationResult<()> {
        self.route_repository.delete(route_id)?;
        self.operation_repository.delete_by_route_id(route_id)?;
        self.segment_repository.delete_by_route_id(route_id)
    }

    pub fn correct_coordinate(&self, coord: &Coordinate) -> ApplicationResult<Coordinate> {
        self.interpolation_api.correct_coordinate(coord)
    }

    fn attach_elevation(&self, seg_list: &mut SegmentList) -> ApplicationResult<()> {
        seg_list
            .iter_mut()
            // `R` cannot be shared between threads safely
            // closureにおそらくselfが入ってるのがいけないんだと思う
            // traitにしたら行けるようになるかも
            // .par_bridge()
            .map(|seg| {
                seg.iter_mut()
                    .filter(|coord| coord.elevation().is_none())
                    .map(|coord| coord.set_elevation(self.elevation_api.get_elevation(coord)?))
                    .try_collect()
            })
            .try_collect()?;
        Ok(())
    }
}
