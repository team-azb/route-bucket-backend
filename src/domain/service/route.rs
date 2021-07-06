use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::domain::model::coordinate::Coordinate;
use crate::domain::model::route::{Route, RouteInfo};
use crate::domain::model::segment::{Segment, SegmentList};
use crate::domain::model::types::RouteId;
use crate::domain::repository::{
    ElevationApi, OperationRepository, RouteInterpolationApi, RouteRepository, SegmentRepository,
};
use crate::utils::error::{ApplicationError, ApplicationResult};

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

    pub fn update_route_info(&self, info: &RouteInfo) -> ApplicationResult<()> {
        self.route_repository.update(info)
    }

    pub fn update_route(&self, route: &Route) -> ApplicationResult<SegmentList> {
        let route_info = route.info();

        self.update_route_info(route_info)?;
        self.operation_repository
            .update_by_route_id(route_info.id(), route.op_list())?;

        if let Some(last_op) = route.last_op() {
            let mut seg_list =
                self.update_segments(route_info.id(), route_info.waypoints(), last_op)?;
            self.attach_elevation(&mut seg_list)?;
            Ok(seg_list)
        } else {
            Err(ApplicationError::DomainError(format!(
                "last_op was None for {}",
                route_info.id().to_string()
            )))
        }
    }

    fn update_segments(
        &self,
        route_id: &RouteId,
        waypoints: &LineString,
        last_op: &Operation,
    ) -> ApplicationResult<SegmentList> {
        match last_op {
            Operation::Add { pos, .. } => {
                self.add_point(route_id, waypoints, *pos)?;
                self.find_segment_list(route_id)
            }
            Operation::Remove { pos, .. } => {
                self.delete_point(route_id, waypoints, *pos)?;
                self.find_segment_list(route_id)
            }
            Operation::Move { pos, .. } => {
                self.move_point(route_id, waypoints, *pos)?;
                self.find_segment_list(route_id)
            }
            Operation::Clear { .. } => {
                self.segment_repository.delete_by_route_id(route_id)?;
                Ok(Vec::new().into())
            }
            Operation::InitWithList { list } => self.insert_waypoints(route_id, list),
        }
    }

    fn add_point(
        &self,
        route_id: &RouteId,
        waypoints: &LineString,
        pos: usize,
    ) -> ApplicationResult<()> {
        let coord = waypoints.get(pos)?;
        let from_opt = (pos > 0).then(|| waypoints.get(pos - 1)).transpose()?;
        let to_opt = waypoints.get(pos + 1).ok();

        if from_opt.is_some() && to_opt.is_some() {
            self.segment_repository.delete(route_id, (pos - 1) as u32)?;
        }
        if let Some(from) = from_opt {
            self.insert_as_segment(route_id, pos - 1, from, coord)?;
        }
        if let Some(to) = to_opt {
            self.insert_as_segment(route_id, pos, coord, to)?;
        }

        Ok(())
    }

    fn delete_point(
        &self,
        route_id: &RouteId,
        waypoints: &LineString,
        pos: usize,
    ) -> ApplicationResult<()> {
        let from_opt = (pos > 0).then(|| waypoints.get(pos - 1)).transpose()?;
        let to_opt = waypoints.get(pos).ok();

        if to_opt.is_some() {
            self.segment_repository.delete(route_id, pos as u32)?;
        }
        if from_opt.is_some() {
            self.segment_repository.delete(route_id, (pos - 1) as u32)?;
        }
        if let Some(from) = from_opt {
            if let Some(to) = to_opt {
                self.insert_as_segment(route_id, pos - 1, from, to)?;
            }
        }

        Ok(())
    }

    fn move_point(
        &self,
        route_id: &RouteId,
        waypoints: &LineString,
        pos: usize,
    ) -> ApplicationResult<()> {
        let coord = waypoints.get(pos)?;
        let from_opt = (pos > 0).then(|| waypoints.get(pos - 1)).transpose()?;
        let to_opt = waypoints.get(pos + 1).ok();

        if let Some(from) = from_opt {
            let seg = self.cvt_to_segment(from, coord)?;
            self.segment_repository
                .update(route_id, (pos - 1) as u32, &seg)?;
        }

        if let Some(to) = to_opt {
            let seg = self.cvt_to_segment(coord, to)?;
            self.segment_repository.update(route_id, pos as u32, &seg)?;
        }

        Ok(())
    }

    fn insert_waypoints(
        &self,
        route_id: &RouteId,
        waypoints: &LineString,
    ) -> ApplicationResult<SegmentList> {
        let seg_list = waypoints
            .iter()
            .tuple_windows()
            // TODO: rayon::par_bridgeで並列化
            .map(|(from, to)| self.cvt_to_segment(&from, &to))
            .collect::<ApplicationResult<Vec<_>>>()?
            .into();

        self.segment_repository
            .insert_by_route_id(route_id, &seg_list)?;

        Ok(seg_list)
    }

    pub fn register_route(&self, route_info: &RouteInfo) -> ApplicationResult<()> {
        self.route_repository.register(route_info)
    }

    pub fn delete_editor(&self, route_id: &RouteId) -> ApplicationResult<()> {
        self.route_repository.delete(route_id)?;
        self.operation_repository.delete_by_route_id(route_id)?;
        self.segment_repository.delete_by_route_id(route_id)
    }

    pub fn correct_coordinate(&self, coord: &Coordinate) -> ApplicationResult<Coordinate> {
        self.interpolation_api.correct_coordinate(coord)
    }

    fn cvt_to_segment(&self, from: &Coordinate, to: &Coordinate) -> ApplicationResult<Segment> {
        let seg = self
            .interpolation_api
            .interpolate(from.clone(), to.clone())?;

        Ok(seg)
    }

    fn insert_as_segment(
        &self,
        route_id: &RouteId,
        pos: usize,
        from: &Coordinate,
        to: &Coordinate,
    ) -> ApplicationResult<()> {
        let seg = self.cvt_to_segment(from, to)?;
        self.segment_repository.insert(route_id, pos as u32, &seg)
    }

    fn attach_elevation(&self, seg_list: &mut SegmentList) -> ApplicationResult<()> {
        seg_list
            .iter_mut()
            .par_bridge()
            .map(|seg| {
                seg.iter_mut()
                    .map(|coord| coord.set_elevation(self.elevation_api.get_elevation(coord)?))
                    .collect::<ApplicationResult<Vec<_>>>()
            })
            .collect::<ApplicationResult<Vec<_>>>()?;
        Ok(())
    }
}
