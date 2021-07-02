use std::convert::TryInto;

use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::model::linestring::{Coordinate, LineString};
use crate::domain::model::operation::OperationStruct;
use crate::domain::model::route::{Route, RouteInfo};
use crate::domain::model::segment::{Segment, SegmentList};
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
        let mut seg_list = self.service.find_segment_list(route_id)?;
        seg_list.attach_distance_from_start()?;
        let elevation_gain = seg_list.calc_elevation_gain()?;

        let (waypoints, segments): (Vec<Coordinate>, Vec<Segment>) = seg_list.into();

        Ok(RouteGetResponse {
            route_info,
            waypoints,
            segments,
            elevation_gain,
        })
    }

    pub fn find_all(&self) -> ApplicationResult<RouteGetAllResponse> {
        Ok(RouteGetAllResponse {
            route_infos: self.service.find_all_routes()?,
        })
    }

    pub fn create(&self, req: &RouteCreateRequest) -> ApplicationResult<RouteCreateResponse> {
        let route_info = RouteInfo::new(RouteId::new(), req.name(), 0);

        self.service.register_route(&route_info)?;

        Ok(RouteCreateResponse {
            id: route_info.id().clone(),
        })
    }

    pub fn rename(&self, route_id: &RouteId, req: &RouteRenameRequest) -> ApplicationResult<Route> {
        let mut route_info = self.service.find_route(route_id)?;
        route_info.rename(req.name());
        self.service.update_route(&route_info)?;
        Ok(route_info)
    }

    pub fn edit(
        &self,
        op_code: &str,
        route_id: &RouteId,
        pos: Option<usize>,
        new_coord: Option<Coordinate>,
    ) -> ApplicationResult<RouteOperationResponse> {
        let mut route = self.service.find_editor(route_id)?;

        let org_polyline = route.route().waypoints().clone();

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
        route.push_operation(opst.try_into()?)?;
        let mut seg_list = self.service.update_editor(&route)?;
        seg_list.attach_distance_from_start()?;
        let elevation_gain = seg_list.calc_elevation_gain()?;

        let (waypoints, segments): (Vec<Coordinate>, Vec<Segment>) = seg_list.into();

        Ok(RouteOperationResponse {
            waypoints,
            segments,
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

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct NewPointRequest {
    coord: Coordinate,
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
