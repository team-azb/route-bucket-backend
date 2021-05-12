use crate::domain::model::linestring::{Coordinate, LineString};
use crate::domain::model::operation::{OperationRepository, OperationStruct};
use crate::domain::model::route::{Route, RouteInterpolationApi, RouteRepository};
use crate::domain::model::types::{Polyline, RouteId};
use crate::domain::service::route::RouteService;
use crate::utils::error::ApplicationResult;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

pub struct RouteUseCase<R, O, I> {
    service: RouteService<R, O, I>,
}

impl<R, O, I> RouteUseCase<R, O, I>
where
    R: RouteRepository,
    O: OperationRepository,
    I: RouteInterpolationApi,
{
    pub fn new(service: RouteService<R, O, I>) -> Self {
        Self { service }
    }

    pub fn find(&self, route_id: &RouteId) -> ApplicationResult<RouteGetResponse> {
        let route = self.service.find_route(route_id)?;
        let polyline = self.service.interpolate_route(&route)?;
        Ok(RouteGetResponse { route, polyline })
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
            new_coord,
            Some(org_polyline),
        )?;
        editor.push_operation(opst.try_into()?)?;
        self.service.update_editor(&editor)?;
        let polyline = self.service.interpolate_route(&editor.route())?;

        Ok(RouteOperationResponse {
            waypoints: editor.route().waypoints().clone(),
            polyline,
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
        let polyline = self.service.interpolate_route(&editor.route())?;

        Ok(RouteOperationResponse {
            waypoints: editor.route().waypoints().clone(),
            polyline,
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
    polyline: Polyline,
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
    pub waypoints: LineString,
    pub polyline: Polyline,
}

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct RouteRenameRequest {
    name: String,
}
