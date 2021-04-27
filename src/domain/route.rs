use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::operation_history::{Operation, OperationHistory};
use crate::domain::polyline::Polyline;
use crate::domain::types::RouteId;
use crate::utils::error::ApplicationResult;

#[derive(Debug, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct Route {
    id: RouteId,
    name: String,
    polyline: Polyline,
    operation_history: OperationHistory,
}

impl Route {
    pub fn new(
        id: RouteId,
        name: &String,
        polyline: Polyline,
        operation_history: OperationHistory,
    ) -> Route {
        Route {
            id,
            name: name.clone(),
            polyline,
            operation_history,
        }
    }

    pub fn rename(&mut self, name: &String) {
        self.name = name.clone();
    }

    pub fn push_operation(&mut self, op: Operation) -> ApplicationResult<()> {
        self.operation_history.push(op, &mut self.polyline)
    }
    pub fn redo_operation(&mut self) -> ApplicationResult<()> {
        self.operation_history.redo(&mut self.polyline)
    }
    pub fn undo_operation(&mut self) -> ApplicationResult<()> {
        self.operation_history.undo(&mut self.polyline)
    }
}

pub trait RouteRepository {
    fn find(&self, id: &RouteId) -> ApplicationResult<Route>;

    fn register(&self, route: &Route) -> ApplicationResult<()>;

    fn update(&self, route: &Route) -> ApplicationResult<()>;
}
