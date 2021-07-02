use crate::domain::model::linestring::{Coordinate, LineString};
use crate::domain::model::operation::OperationStruct;
use crate::domain::model::route::Route;
use crate::domain::model::segment::SegmentList;
use crate::domain::model::types::{Elevation, RouteId};
use crate::domain::repository::{
    ElevationApi, OperationRepository, RouteInterpolationApi, RouteRepository, SegmentRepository,
};
use crate::domain::service::route::RouteService;
use crate::utils::error::ApplicationResult;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

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
        let route = self.service.find_route(route_id)?;
        let mut seg_list = self.service.find_segment_list(route_id)?;
        seg_list.attach_distance_from_start()?;
        let elevation_gain = seg_list.calc_elevation_gain()?;
        Ok(RouteGetResponse {
            route,
            segments: seg_list,
            elevation_gain,
        })
    }

    pub fn find_all(&self) -> ApplicationResult<RouteGetAllResponse> {
        Ok(RouteGetAllResponse {
            routes: self.service.find_all_routes()?,
        })
    }

    pub fn create(&self, req: &RouteCreateRequest) -> ApplicationResult<RouteCreateResponse> {
        let route = Route::new(RouteId::new(), req.name(), LineString::new(), 0);

        self.service.register_route(&route)?;

        Ok(RouteCreateResponse {
            id: route.id().clone(),
        })
    }

    pub fn rename(&self, route_id: &RouteId, req: &RouteRenameRequest) -> ApplicationResult<Route> {
        let mut route = self.service.find_route(route_id)?;
        route.rename(req.name());
        self.service.update_route(&route)?;
        Ok(route)
    }

    pub fn edit(
        &self,
        op_code: &str,
        route_id: &RouteId,
        pos: Option<usize>,
        new_coord: Option<Coordinate>,
    ) -> ApplicationResult<RouteOperationResponse> {
        let mut editor = self.service.find_editor(route_id)?;

        let org_polyline = editor.route().waypoints().clone();

        let org_coord = pos.map_or(None, |pos| {
            org_polyline.get(pos).ok().map(Coordinate::clone)
        });

        let opst = OperationStruct::new(
            op_code.into(),
            pos,
            org_coord,
            // TODO: 道路外でも許容する場合（直線モードとか？）としない場合の区別をする
            new_coord
                .map(|coord| self.service.correct_coordinate(&coord))
                .transpose()?,
            Some(org_polyline),
        )?;
        editor.push_operation(opst.try_into()?)?;
        let mut seg_list = self.service.update_editor(&editor)?;
        seg_list.attach_distance_from_start()?;
        let elevation_gain = seg_list.calc_elevation_gain()?;

        Ok(RouteOperationResponse {
            waypoints: editor.route().waypoints().clone(),
            segments: seg_list,
            elevation_gain,
        })
    }

    pub fn migrate_history(
        &self,
        route_id: &RouteId,
        forward: bool,
    ) -> ApplicationResult<RouteOperationResponse> {
        let mut editor = self.service.find_editor(route_id)?;
        if forward {
            editor.redo_operation()?;
        } else {
            editor.undo_operation()?;
        }
        let segments = self.service.update_editor(&editor)?;
        let elevation_gain = segments.calc_elevation_gain()?;

        Ok(RouteOperationResponse {
            waypoints: editor.route().waypoints().clone(),
            segments,
            elevation_gain,
        })
    }

    pub fn delete(&self, route_id: &RouteId) -> ApplicationResult<()> {
        self.service.delete_editor(route_id)
    }
}

#[derive(Serialize)]
pub struct RouteGetResponse {
    #[serde(flatten)]
    route: Route,
    segments: SegmentList,
    elevation_gain: Elevation,
}

#[derive(Serialize)]
pub struct RouteGetAllResponse {
    routes: Vec<Route>,
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

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct NewPointRequest {
    coord: Coordinate,
}

#[derive(Serialize)]
pub struct RouteOperationResponse {
    waypoints: LineString,
    segments: SegmentList,
    elevation_gain: Elevation,
}

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct RouteRenameRequest {
    name: String,
}
