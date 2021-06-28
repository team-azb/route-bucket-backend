use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::model::linestring::{Coordinate, LineString};
use crate::domain::model::operation::Operation;
use crate::domain::model::types::{Polyline, RouteId};
use crate::utils::error::{ApplicationError, ApplicationResult};

#[derive(Debug, Getters)]
#[get = "pub"]
pub struct RouteEditor {
    route: Route,
    op_list: Vec<Operation>,
}

impl RouteEditor {
    pub fn new(route: Route, op_list: Vec<Operation>) -> Self {
        Self { route, op_list }
    }

    pub fn push_operation(&mut self, op: Operation) -> ApplicationResult<()> {
        // pos以降の要素は全て捨てる
        self.op_list.truncate(self.route.op_num);
        self.op_list.push(op);

        self.apply_operation(false)
    }

    pub fn redo_operation(&mut self) -> ApplicationResult<()> {
        if self.route.op_num < self.op_list.len() {
            self.apply_operation(false)
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to redo.",
            ))
        }
    }

    pub fn undo_operation(&mut self) -> ApplicationResult<()> {
        if self.route.op_num > 0 {
            self.apply_operation(true)
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to undo.",
            ))
        }
    }

    fn apply_operation(&mut self, reverse: bool) -> ApplicationResult<()> {
        let op;
        if reverse {
            self.route.op_num -= 1;
            op = self.get_operation(self.route.op_num)?.reverse();
        } else {
            op = self.get_operation(self.route.op_num)?.clone();
            self.route.op_num += 1;
        };

        op.apply(&mut self.route.waypoints)?;
        self.last_op.insert(op);

        Ok(())
    }
}

#[derive(Debug, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct Route {
    id: RouteId,
    name: String,
    waypoints: LineString,
    #[serde(skip_serializing)]
    op_num: usize,
}

impl Route {
    pub fn new(id: RouteId, name: &String, waypoints: LineString, op_num: usize) -> Route {
        Route {
            id,
            name: name.clone(),
            waypoints,
            op_num,
        }
    }

    pub fn rename(&mut self, name: &String) {
        self.name = name.clone();
    }
}
