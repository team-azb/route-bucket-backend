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
    // TODO: DBにはPolylineとして保存する
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

    pub fn add_operation(&mut self, op: Operation) {
        self.operation_history.add(op);
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
}
