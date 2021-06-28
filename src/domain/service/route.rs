use itertools::Itertools;

use crate::domain::model::linestring::{Coordinate, LineString};
use crate::domain::model::operation::Operation;
use crate::domain::model::route::{Route, RouteEditor};
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

    pub fn find_route(&self, route_id: &RouteId) -> ApplicationResult<Route> {
        self.route_repository.find(route_id)
    }

    pub fn find_all_routes(&self) -> ApplicationResult<Vec<Route>> {
        self.route_repository.find_all()
    }

    pub fn find_editor(&self, route_id: &RouteId) -> ApplicationResult<RouteEditor> {
        let route = self.find_route(route_id)?;
        let operations = self.operation_repository.find_by_route_id(route_id)?;

        Ok(RouteEditor::new(route, operations))
    }

    pub fn find_segments(&self, route_id: &RouteId) -> ApplicationResult<SegmentList> {
        let mut segments = self.segment_repository.find_by_id(route_id)?;
        self.attach_elevation(&mut segments)?;
        Ok(segments)
    }

    pub fn update_route(&self, route: &Route) -> ApplicationResult<()> {
        self.route_repository.update(route)
    }

    pub fn update_editor(&self, editor: &RouteEditor) -> ApplicationResult<SegmentList> {
        let route = editor.route();

        self.update_route(route)?;
        self.operation_repository
            .update_by_route_id(route.id(), editor.op_list())?;

        if let Some(last_op) = editor.last_op() {
            let mut segments = self.update_segments(route.id(), route.waypoints(), last_op)?;
            self.attach_elevation(&mut segments)?;
            Ok(segments)
        } else {
            Err(ApplicationError::DomainError(format!(
                "last_op was None for {}",
                route.id().to_string()
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
                self.find_segments(route_id)
            }
            Operation::Remove { pos, .. } => {
                self.delete_point(route_id, waypoints, *pos)?;
                self.find_segments(route_id)
            }
            Operation::Move { pos, .. } => {
                self.move_point(route_id, waypoints, *pos)?;
                self.find_segments(route_id)
            }
            Operation::Clear { .. } => {
                self.segment_repository.delete_by_id(route_id)?;
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
            let segment = self.cvt_to_segment(from, coord)?;
            println!("{:?}, {:?}, {}", from, coord, pos - 1);
            self.segment_repository
                .update(route_id, (pos - 1) as u32, &segment)?;
        }

        if let Some(to) = to_opt {
            let segment = self.cvt_to_segment(coord, to)?;
            println!("{:?}, {:?}, {}", coord, to, pos);
            self.segment_repository
                .update(route_id, pos as u32, &segment)?;
        }

        Ok(())
    }

    fn insert_waypoints(
        &self,
        route_id: &RouteId,
        waypoints: &LineString,
    ) -> ApplicationResult<SegmentList> {
        let segments = waypoints
            .iter()
            .tuple_windows()
            // TODO: rayon::par_bridgeで並列化
            .map(|(from, to)| self.cvt_to_segment(&from, &to))
            .collect::<ApplicationResult<Vec<_>>>()?
            .into();

        self.segment_repository.insert_by_id(route_id, &segments)?;

        Ok(segments)
    }

    pub fn register_route(&self, route: &Route) -> ApplicationResult<()> {
        self.route_repository.register(route)
    }

    pub fn delete_editor(&self, route_id: &RouteId) -> ApplicationResult<()> {
        self.route_repository.delete(route_id)?;
        self.operation_repository.delete_by_route_id(route_id)?;
        self.segment_repository.delete_by_id(route_id)
    }

    pub fn correct_coordinate(&self, coord: &Coordinate) -> ApplicationResult<Coordinate> {
        self.interpolation_api.correct_coordinate(coord)
    }

    fn cvt_to_segment(&self, from: &Coordinate, to: &Coordinate) -> ApplicationResult<Segment> {
        let segment = self
            .interpolation_api
            .interpolate(from.clone(), to.clone())?;

        Ok(segment)
    }

    fn insert_as_segment(
        &self,
        route_id: &RouteId,
        pos: usize,
        from: &Coordinate,
        to: &Coordinate,
    ) -> ApplicationResult<()> {
        let segment = self.cvt_to_segment(from, to)?;
        self.segment_repository
            .insert(route_id, pos as u32, &segment)
    }

    fn attach_elevation(&self, seg_list: &mut SegmentList) -> ApplicationResult<()> {
        seg_list
            .iter_mut()
            .map(|seg| {
                seg.iter_mut()
                    .map(|coord| coord.set_elevation(self.elevation_api.get_elevation(coord)?))
                    .collect::<ApplicationResult<Vec<_>>>()
            })
            .collect::<ApplicationResult<Vec<_>>>()?;
        Ok(())
    }
}
