use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::model::operation::{Operation, OperationRepository};
use crate::domain::model::polyline::{Coordinate, Polyline};
use crate::domain::model::route::{Route, RouteRepository};
use crate::domain::model::types::RouteId;
use crate::domain::service::route::RouteService;
use crate::utils::error::{ApplicationError, ApplicationResult};

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
        coord: Option<Coordinate>,
    ) -> ApplicationResult<RouteOperationResponse> {
        let mut editor = self.service.find_editor(route_id)?;

        let coord_vec = coord.map_or(vec![], |coord| vec![coord]);
        let pos_vec = pos.map_or(Ok(vec![]), |pos| {
            Ok(vec![editor.route().polyline().get(pos)?.clone()])
        })?;

        let op_polyline = match op_code {
            "add" => Polyline::from_vec(coord_vec),
            "rm" => Polyline::from_vec(pos_vec),
            "mv" => Polyline::from_vec([pos_vec, coord_vec].concat()),
            "clear" => editor.route().polyline().clone(),
            _ => {
                return Err(ApplicationError::UseCaseError(format!(
                    "edit for op_code {} isn't implemented!",
                    op_code
                )))
            }
        };

        let op = Operation::from_code(
            &String::from(op_code),
            pos.map(|i| i as u32),
            &op_polyline.encode()?,
        )?;
        editor.push_operation(op)?;
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
