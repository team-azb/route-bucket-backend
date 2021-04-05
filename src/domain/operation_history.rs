use serde::{Deserialize, Serialize};

use crate::domain::polyline::{Coordinate, Polyline};
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

    pub fn undo(&mut self, polyline: &mut Polyline) -> ApplicationResult<()> {
        if self.pos > 0 {
            self.pos -= 1;
            self.operations[self.pos].reverse().apply(polyline)?;
            Ok(())
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to undo.",
            ))
        }
    }

    pub fn redo(&mut self, polyline: &mut Polyline) -> ApplicationResult<()> {
        if self.pos < self.operations.len() {
            self.operations[self.pos].apply(polyline)?;
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
    Clear { org_list: Polyline },
    // reverse operation for Clear
    InitWithList { list: Polyline },
}

impl Operation {
    pub fn apply(&self, polyline: &mut Polyline) -> ApplicationResult<()> {
        match self {
            Self::Add { pos, coord } => Ok(polyline.insert_point(*pos, coord.clone())?),
            Self::Remove { pos, coord } => {
                let ref removed = polyline.remove_point(*pos)?;
                (coord == removed)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError("Contradiction on remove"))
            }
            Self::Clear { org_list } => {
                let ref removed_list = polyline.clear_points();
                (org_list == removed_list)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError("Contradiction on clear"))
            }
            Self::InitWithList { list } => Ok(polyline.init_points(list.clone())?),
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
