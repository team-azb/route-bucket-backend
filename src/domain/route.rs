use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::coordinate::Coordinate;
use crate::domain::operation_history::{Operation, OperationHistory};
use crate::domain::types::RouteId;
use crate::utils::error::{ApplicationError, ApplicationResult};

#[derive(Debug, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct Route {
    id: RouteId,
    name: String,
    // TODO: DBにはPolylineとして保存する
    points: Vec<Coordinate>,
    operation_history: OperationHistory,
}

impl Route {
    pub fn new(
        id: RouteId,
        name: &String,
        points: Vec<Coordinate>,
        // operation_history: OperationHistory,
    ) -> Route {
        Route {
            id,
            name: name.clone(),
            points,
            operation_history: OperationHistory::new(),
        }
    }

    pub fn add_operation(&mut self, op: Operation) {
        self.operation_history.add(op);
    }
    pub fn redo_operation(&mut self) -> ApplicationResult<()> {
        self.operation_history.redo(self)
    }
    pub fn undo_operation(&mut self) -> ApplicationResult<()> {
        self.operation_history.undo(self)
    }

    // TODO: この辺はPolylineに持たせる
    pub fn insert_point(&mut self, pos: usize, point: Coordinate) -> ApplicationResult<()> {
        if pos > self.points.len() {
            // TODO: ここの説明の改善
            Err(ApplicationError::DomainError("Failed to insert point."))
        } else {
            Ok(self.points.insert(pos, point))
        }
    }

    pub fn remove_point(&mut self, pos: usize) -> ApplicationResult<Coordinate> {
        if pos > self.points.len() {
            Err(ApplicationError::DomainError("Failed to remove point."))
        } else {
            Ok(self.points.remove(pos))
        }
    }

    pub fn clear_points(&mut self) -> Vec<Coordinate> {
        std::mem::replace(&mut self.points, Vec::new())
    }

    // only when points is empty
    pub fn init_points(&mut self, points: Vec<Coordinate>) -> ApplicationResult<()> {
        if self.points.is_empty() {
            Err(ApplicationError::DomainError(
                "Failed to set points. self.points was already inited.",
            ))
        } else {
            self.points = points;
            Ok(())
        }
    }
}

pub trait RouteRepository {
    fn find(&self, id: &RouteId) -> ApplicationResult<Route>;

    fn register(&self, route: &Route) -> ApplicationResult<()>;
}
