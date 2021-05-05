use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::model::operation::Operation;
use crate::domain::model::polyline::Polyline;
use crate::domain::model::types::RouteId;
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
        op.apply(&mut self.route.polyline)?;
        // pos以降の要素は全て捨てる
        self.op_list.truncate(self.route.op_num);
        self.op_list.push(op);
        self.route.op_num += 1;
        Ok(())
    }

    pub fn redo_operation(&mut self) -> ApplicationResult<()> {
        if self.route.op_num < self.op_list.len() {
            self.op_list[self.route.op_num].apply(&mut self.route.polyline)?;
            self.route.op_num += 1;
            Ok(())
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to redo.",
            ))
        }
    }

    pub fn undo_operation(&mut self) -> ApplicationResult<()> {
        if self.route.op_num > 0 {
            self.route.op_num -= 1;
            self.op_list[self.route.op_num]
                .reverse()
                .apply(&mut self.route.polyline)?;
            Ok(())
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to undo.",
            ))
        }
    }
}

#[derive(Debug, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct Route {
    id: RouteId,
    name: String,
    polyline: Polyline,
    op_num: usize,
}

impl Route {
    pub fn new(id: RouteId, name: &String, polyline: Polyline, op_num: usize) -> Route {
        Route {
            id,
            name: name.clone(),
            polyline,
            op_num,
        }
    }

    pub fn rename(&mut self, name: &String) {
        self.name = name.clone();
    }
}

pub trait RouteRepository {
    fn find(&self, id: &RouteId) -> ApplicationResult<Route>;

    fn find_all(&self) -> ApplicationResult<Vec<Route>>;

    fn register(&self, route: &Route) -> ApplicationResult<()>;

    fn update(&self, route: &Route) -> ApplicationResult<()>;

    fn delete(&self, id: &RouteId) -> ApplicationResult<()>;
}
