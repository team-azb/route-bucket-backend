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

    pub fn edit(
        &self,
        op_code: &str,
        route_id: &RouteId,
        pos: Option<usize>,
        coord: Option<Coordinate>,
    ) -> ApplicationResult<Route> {
        let mut route = self.repository.find(route_id)?;
        let op_polyline = match op_code {
            "add" => {
                let vec = coord.map_or(vec![], |coord| vec![coord]);
                Polyline::from_vec(vec)
            }
            "rm" => {
                let vec = pos.map_or(vec![], |pos| vec![route.polyline()[pos].clone()]);
                Polyline::from_vec(vec)
            }
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
        route.push_operation(op);
        self.repository.update(&route);

        Ok(route)
    }

    pub fn migrate_history(&self, route_id: &RouteId, forward: bool) -> ApplicationResult<Route> {
        let mut route = self.repository.find(route_id)?;
        if forward {
            route.redo_operation()?;
        } else {
            route.undo_operation()?;
        }
        self.repository.update(&route);

        Ok(route)
    }
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
pub struct AddPointRequest {
    coord: Coordinate,
}
