use crate::domain::polyline::{Coordinate, Polyline};
use crate::utils::error::{ApplicationError, ApplicationResult};
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct OperationHistory {
    op_list: Vec<Operation>,
    pos: usize,
}

impl OperationHistory {
    pub fn new(op_list: Vec<Operation>, pos: usize) -> Self {
        Self { op_list, pos }
    }

    pub fn push(&mut self, op: Operation, polyline: &mut Polyline) -> ApplicationResult<()> {
        op.apply(polyline)?;
        // pos以降の要素は全て捨てる
        self.op_list.truncate(self.pos);
        self.op_list.push(op);
        self.pos += 1;
        Ok(())
    }

    pub fn undo(&mut self, polyline: &mut Polyline) -> ApplicationResult<()> {
        if self.pos > 0 {
            self.pos -= 1;
            self.op_list[self.pos].reverse().apply(polyline)?;
            Ok(())
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to undo.",
            ))
        }
    }

    pub fn redo(&mut self, polyline: &mut Polyline) -> ApplicationResult<()> {
        if self.pos < self.op_list.len() {
            self.op_list[self.pos].apply(polyline)?;
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
    Add {
        pos: u32,
        coord: Coordinate,
    },
    Remove {
        pos: u32,
        coord: Coordinate,
    },
    Move {
        pos: u32,
        from: Coordinate,
        to: Coordinate,
    },
    Clear {
        org_list: Polyline,
    },
    // reverse operation for Clear
    InitWithList {
        list: Polyline,
    },
}

impl Operation {
    pub fn to_code(&self) -> &str {
        match self {
            Self::Add { .. } => "add",
            Self::Remove { .. } => "rm",
            Self::Move { .. } => "mv",
            Self::Clear { .. } => "clear",
            Self::InitWithList { .. } => "init",
        }
    }

    pub fn from_code(
        code: &String,
        pos: Option<u32>,
        polyline: &String,
    ) -> ApplicationResult<Operation> {
        let coord_list = Polyline::decode(polyline)?;
        let op = match &**code {
            "add" | "rm" | "mv" => {
                let pos = pos.ok_or(ApplicationError::DomainError(format!(
                    "invalid operation {} without pos",
                    code
                )))?;
                match &**code {
                    "add" => Operation::Add {
                        pos,
                        coord: coord_list.get(0)?.clone(),
                    },
                    "rm" => Operation::Remove {
                        pos,
                        coord: coord_list.get(0)?.clone(),
                    },
                    // "mv"
                    _ => Operation::Move {
                        pos,
                        from: coord_list.get(0)?.clone(),
                        to: coord_list.get(1)?.clone(),
                    },
                }
            }
            "clear" => Operation::Clear {
                org_list: coord_list,
            },
            "init" => Operation::InitWithList { list: coord_list },
            _ => {
                return Err(ApplicationError::DomainError(format!(
                    "invalid operation code {}",
                    code
                )))
            }
        };
        Ok(op)
    }

    pub fn apply(&self, polyline: &mut Polyline) -> ApplicationResult<()> {
        match self {
            Self::Add { pos, coord } => Ok(polyline.insert(*pos as usize, coord.clone())?),
            Self::Remove { pos, coord } => {
                let ref removed = polyline.remove(*pos as usize)?;
                (coord == removed)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError(
                        "Contradiction on remove".into(),
                    ))
            }
            Self::Move { pos, from, to } => {
                let ref moved = polyline.replace(*pos as usize, to.clone())?;
                (from == moved)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError(
                        "Contradiction on move".into(),
                    ))
            }
            Self::Clear { org_list } => {
                let ref removed_list = polyline.clear();
                (org_list == removed_list)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError(
                        "Contradiction on clear".into(),
                    ))
            }
            Self::InitWithList { list } => Ok(polyline.init_if_empty(list.clone())?),
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
            Self::Move { pos, from, to } => Self::Move {
                pos: *pos,
                from: to.clone(),
                to: from.clone(),
            },
            Self::Clear { org_list: list } => Self::InitWithList { list: list.clone() },
            Self::InitWithList { list } => Self::Clear {
                org_list: list.clone(),
            },
        }
    }
}
