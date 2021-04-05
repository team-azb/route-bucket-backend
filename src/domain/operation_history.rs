use serde::{Deserialize, Serialize};

use crate::domain::coordinate::Coordinate;
use crate::domain::route::Route;
use crate::utils::error::{ApplicationError, ApplicationResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct OperationHistory {
    operations: Vec<Operation>,
    pos: usize,
}

impl OperationHistory {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            pos: 0,
        }
    }

    pub fn add(&mut self, op: Operation) {
        // pos以降の要素は全て捨てる
        self.operations.truncate(self.pos);
        self.operations.push(op);
        self.pos += 1;
    }

    pub fn undo(&mut self, route: &mut Route) -> ApplicationResult<()> {
        if self.pos > 0 {
            self.pos -= 1;
            self.operations[self.pos].reverse().apply(route)?;
            Ok(())
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to undo.",
            ))
        }
    }

    pub fn redo(&mut self, route: &mut Route) -> ApplicationResult<()> {
        if self.pos < self.operations.len() {
            self.operations[self.pos].apply(route)?;
            self.pos += 1;
            Ok(())
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to redo.",
            ))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Operation {
    Add { pos: usize, coord: Coordinate },
    Remove { pos: usize, coord: Coordinate },
    Clear { org_list: Vec<Coordinate> },
    // reverse operation for Clear
    InitWithList { list: Vec<Coordinate> },
}

impl Operation {
    // TODO: Polylineを実装したらinsert_pointとかをPolylineのメソッドにして、
    //     : この引数もRouteではなくPolylineにする
    pub fn apply(&self, route: &mut Route) -> ApplicationResult<()> {
        match self {
            Self::Add { pos, coord } => Ok(route.insert_point(*pos, coord.clone())?),
            Self::Remove { pos, coord } => {
                let ref removed = route.remove_point(*pos)?;
                (coord == removed)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError("Contradiction on remove"))
            }
            Self::Clear { org_list } => {
                let ref removed_list = route.clear_points();
                (org_list == removed_list)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError("Contradiction on clear"))
            }
            Self::InitWithList { list } => Ok(route.init_points(list.clone())?),
        }
    }

    pub fn reverse(&self) -> Operation {
        match self {
            Self::Add { pos, coord } => Self::Remove {
                pos: *pos,
                coord: coord.clone(),
            },
            Self::Remove { pos, coord } => Self::Add {
                pos: *pos,
                coord: coord.clone(),
            },
            Self::Clear { org_list } => Self::InitWithList {
                list: org_list.clone(),
            },
            Self::InitWithList { list } => Self::Clear {
                org_list: list.clone(),
            },
        }
    }
}
