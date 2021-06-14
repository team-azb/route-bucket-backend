use crate::domain::model::linestring::{Coordinate, ElevationApi, LineString};
use crate::domain::model::operation::{OperationRepository, OperationStruct};
use crate::domain::model::route::{Route, RouteInterpolationApi, RouteRepository};
use crate::domain::model::types::{Elevation, RouteId};
use crate::domain::service::route::RouteService;
use crate::utils::error::ApplicationResult;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

pub struct RouteUseCase<R, O, I, E> {
    service: RouteService<R, O, I, E>,
}

impl<R, O, I, E> RouteUseCase<R, O, I, E>
where
    R: RouteRepository,
    O: OperationRepository,
    I: RouteInterpolationApi,
    E: ElevationApi,
{
    pub fn new(service: RouteService<R, O, I, E>) -> Self {
        Self { service }
    }

    pub fn find(&self, route_id: &RouteId) -> ApplicationResult<RouteGetResponse> {
        let route = self.service.find_route(route_id)?;
        let linestring = self.service.interpolate_route(&route)?;
        let elevation_gain = linestring.calc_elevation_gain()?;
        Ok(RouteGetResponse {
            route,
            linestring,
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
        self.service.update_editor(&editor)?;
        let linestring = self.service.interpolate_route(&editor.route())?;
        let elevation_gain = linestring.calc_elevation_gain()?;

        Ok(RouteOperationResponse {
            waypoints: editor.route().waypoints().clone(),
            linestring,
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
        self.service.update_route(&editor.route())?;
        let linestring = self.service.interpolate_route(&editor.route())?;
        let elevation_gain = linestring.calc_elevation_gain()?;

        Ok(RouteOperationResponse {
            waypoints: editor.route().waypoints().clone(),
            linestring,
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
    linestring: LineString,
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
    linestring: LineString,
    elevation_gain: Elevation,
}

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct RouteRenameRequest {
    name: String,
}
