use crate::domain::model::operation::{OperationRepository, OperationStruct};
use crate::domain::model::polyline::{Coordinate, Polyline};
use crate::domain::model::route::{Route, RouteRepository};
use crate::domain::model::types::RouteId;
use crate::domain::service::route::RouteService;
use crate::utils::error::ApplicationResult;
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

pub struct RouteUseCase<R: RouteRepository, O: OperationRepository> {
    service: RouteService<R, O>,
}

impl<R: RouteRepository, O: OperationRepository> RouteUseCase<R, O> {
    pub fn new(service: RouteService<R, O>) -> Self {
        Self { service }
    }

    pub fn find(&self, route_id: &RouteId) -> ApplicationResult<Route> {
        self.service.find_route(route_id)
    }

    pub fn find_all(&self) -> ApplicationResult<RouteGetAllRequest> {
        Ok(RouteGetAllRequest {
            routes: self.service.find_all_routes()?,
        })
    }

    pub fn create(&self, req: &RouteCreateRequest) -> ApplicationResult<RouteCreateResponse> {
        let route = Route::new(RouteId::new(), req.name(), Polyline::new(), 0);

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

        let org_polyline = editor.route().polyline().clone();

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

        Ok(RouteOperationResponse {
            polyline: editor.route().polyline().clone(),
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

        Ok(RouteOperationResponse {
            polyline: editor.route().polyline().clone(),
        })
    }

    pub fn delete(&self, route_id: &RouteId) -> ApplicationResult<()> {
        self.service.delete_editor(route_id)
    }
}

#[derive(Serialize)]
pub struct RouteGetAllRequest {
    pub routes: Vec<Route>,
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
    pub polyline: Polyline,
}

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct RouteRenameRequest {
    name: String,
}
