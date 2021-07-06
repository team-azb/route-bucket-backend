use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::model::coordinate::Coordinate;
use crate::domain::model::operation::Operation;
use crate::domain::model::route::{Route, RouteInfo};
use crate::domain::model::segment::Segment;
use crate::domain::model::types::{Elevation, RouteId};
use crate::domain::repository::{
    ElevationApi, OperationRepository, RouteInterpolationApi, RouteRepository, SegmentRepository,
};
use crate::domain::service::route::RouteService;
use crate::utils::error::ApplicationResult;

pub struct RouteUseCase<R, O, S, I, E> {
    service: RouteService<R, O, S, I, E>,
}

impl<R, O, S, I, E> RouteUseCase<R, O, S, I, E>
where
    R: RouteRepository,
    O: OperationRepository,
    S: SegmentRepository,
    I: RouteInterpolationApi,
    E: ElevationApi,
{
    pub fn new(service: RouteService<R, O, S, I, E>) -> Self {
        Self { service }
    }

    pub fn find(&self, route_id: &RouteId) -> ApplicationResult<RouteGetResponse> {
        let route_info = self.service.find_info(route_id)?;
        let seg_list = self.service.find_segment_list(route_id)?;
        let elevation_gain = seg_list.calc_elevation_gain()?;

        let waypoints = seg_list.gather_waypoints();
        let segments = seg_list.into_segments_in_between();

        Ok(RouteGetResponse {
            route_info,
            waypoints,
            segments,
            elevation_gain,
        })
    }

    pub fn find_all(&self) -> ApplicationResult<RouteGetAllResponse> {
        Ok(RouteGetAllResponse {
            route_infos: self.service.find_all_infos()?,
        })
    }

    pub fn create(&self, req: &RouteCreateRequest) -> ApplicationResult<RouteCreateResponse> {
        let route_info = RouteInfo::new(RouteId::new(), req.name(), 0);

        self.service.register_route(&route_info)?;

        Ok(RouteCreateResponse {
            id: route_info.id().clone(),
        })
    }

    pub fn rename(
        &self,
        route_id: &RouteId,
        req: &RouteRenameRequest,
    ) -> ApplicationResult<RouteInfo> {
        let mut route_info = self.service.find_info(route_id)?;
        route_info.rename(req.name());
        self.service.update_info(&route_info)?;
        Ok(route_info)
    }

    pub fn add_point(
        &self,
        route_id: &RouteId,
        pos: usize,
        coord: Coordinate,
    ) -> ApplicationResult<RouteOperationResponse> {
        let mut route = self.service.find_route(route_id)?;
        route.push_operation(Operation::new_add(pos, coord))?;
        self.update(&mut route)
    }

    pub fn remove_point(
        &self,
        route_id: &RouteId,
        pos: usize,
    ) -> ApplicationResult<RouteOperationResponse> {
        let mut route = self.service.find_route(route_id)?;
        route.push_operation(Operation::new_remove(pos, route.gather_waypoints()))?;
        self.update(&mut route)
    }

    pub fn move_point(
        &self,
        route_id: &RouteId,
        pos: usize,
        coord: Coordinate,
    ) -> ApplicationResult<RouteOperationResponse> {
        let mut route = self.service.find_route(route_id)?;
        route.push_operation(Operation::new_move(pos, coord, route.gather_waypoints()))?;
        self.update(&mut route)
    }

    pub fn clear_route(&self, route_id: &RouteId) -> ApplicationResult<RouteOperationResponse> {
        let mut route = self.service.find_route(route_id)?;
        route.push_operation(Operation::new_clear(route.gather_waypoints()))?;
        self.update(&mut route)
    }

    pub fn redo_operation(&self, route_id: &RouteId) -> ApplicationResult<RouteOperationResponse> {
        let mut route = self.service.find_route(route_id)?;
        route.redo_operation()?;
        self.update(&mut route)
    }

    pub fn undo_operation(&self, route_id: &RouteId) -> ApplicationResult<RouteOperationResponse> {
        let mut route = self.service.find_route(route_id)?;
        route.undo_operation()?;
        self.update(&mut route)
    }

    pub fn delete(&self, route_id: &RouteId) -> ApplicationResult<()> {
        self.service.delete_route(route_id)
    }

    fn update(&self, route: &mut Route) -> ApplicationResult<RouteOperationResponse> {
        // TODO: posのrangeチェック

        self.service.update_route(route)?;
        let elevation_gain = route.calc_elevation_gain()?;

        // NOTE: どうせここでcloneが必要なので、update_routeの戻り値にSegmentListを指定してもいいかもしれない
        let seg_list = route.seg_list().clone();
        let waypoints = seg_list.gather_waypoints();
        let segments = seg_list.into_segments_in_between();

        Ok(RouteOperationResponse {
            waypoints,
            segments,
            elevation_gain,
        })
    }
}

#[derive(Serialize)]
pub struct RouteGetResponse {
    #[serde(flatten)]
    route_info: RouteInfo,
    waypoints: Vec<Coordinate>,
    segments: Vec<Segment>,
    elevation_gain: Elevation,
}

#[derive(Serialize)]
pub struct RouteGetAllResponse {
    #[serde(rename = "routes")]
    route_infos: Vec<RouteInfo>,
}

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct RouteCreateRequest {
    name: String,
}

#[derive(Serialize)]
pub struct RouteCreateResponse {
    pub id: RouteId,
}

#[derive(Deserialize)]
pub struct NewPointRequest {
    pub coord: Coordinate,
}

#[derive(Serialize)]
pub struct RouteOperationResponse {
    waypoints: Vec<Coordinate>,
    segments: Vec<Segment>,
    elevation_gain: Elevation,
}

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct RouteRenameRequest {
    name: String,
}
