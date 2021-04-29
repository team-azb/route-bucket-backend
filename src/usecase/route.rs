use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::operation_history::{Operation, OperationHistory};
use crate::domain::polyline::{Coordinate, Polyline};
use crate::domain::route::{Route, RouteRepository};
use crate::domain::types::RouteId;
use crate::utils::error::{ApplicationError, ApplicationResult};

pub struct RouteUseCase<R: RouteRepository> {
    repository: R,
}

impl<R: RouteRepository> RouteUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn find(&self, route_id: &RouteId) -> ApplicationResult<Route> {
        self.repository.find(route_id)
    }

    pub fn find_all(&self) -> ApplicationResult<RouteGetAllRequest> {
        Ok(RouteGetAllRequest {
            routes: self.repository.find_all()?,
        })
    }

    pub fn create(&self, req: &RouteCreateRequest) -> ApplicationResult<RouteCreateResponse> {
        let route = Route::new(
            RouteId::new(),
            req.name(),
            Polyline::new(),
            OperationHistory::new(vec![], 0),
        );

        self.repository.register(&route)?;

        Ok(RouteCreateResponse {
            id: route.id().clone(),
        })
    }

    pub fn rename(&self, route_id: &RouteId, req: &RouteRenameRequest) -> ApplicationResult<Route> {
        let mut route = self.repository.find(route_id)?;
        route.rename(req.name());
        self.repository.update(&route)?;
        Ok(route)
    }

    pub fn edit(
        &self,
        op_code: &str,
        route_id: &RouteId,
        pos: Option<usize>,
        coord: Option<Coordinate>,
    ) -> ApplicationResult<RouteOperationResponse> {
        let mut route = self.repository.find(route_id)?;

        let coord_vec = coord.map_or(vec![], |coord| vec![coord]);
        let pos_vec = pos.map_or(Ok(vec![]), |pos| {
            Ok(vec![route.polyline().get(pos)?.clone()])
        })?;

        let op_polyline = match op_code {
            "add" => Polyline::from_vec(coord_vec),
            "rm" => Polyline::from_vec(pos_vec),
            "mv" => Polyline::from_vec([pos_vec, coord_vec].concat()),
            "clear" => route.polyline().clone(),
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
        route.push_operation(op)?;
        self.repository.update(&route)?;

        Ok(RouteOperationResponse {
            points: route.polyline().clone(),
        })
    }

    pub fn migrate_history(
        &self,
        route_id: &RouteId,
        forward: bool,
    ) -> ApplicationResult<RouteOperationResponse> {
        let mut route = self.repository.find(route_id)?;
        if forward {
            route.redo_operation()?;
        } else {
            route.undo_operation()?;
        }
        self.repository.update(&route)?;

        Ok(RouteOperationResponse {
            points: route.polyline().clone(),
        })
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
    pub points: Polyline,
}

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct RouteRenameRequest {
    name: String,
}
