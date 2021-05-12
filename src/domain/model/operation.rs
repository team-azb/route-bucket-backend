use crate::domain::model::linestring::{Coordinate, LineString};
use crate::domain::model::types::RouteId;
use crate::utils::error::{ApplicationError, ApplicationResult};
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Clone, Debug)]
pub enum Operation {
    Add {
        pos: usize,
        coord: Coordinate,
    },
    Remove {
        pos: usize,
        coord: Coordinate,
    },
    Move {
        pos: usize,
        from: Coordinate,
        to: Coordinate,
    },
    Clear {
        org_list: LineString,
    },
    // reverse operation for Clear
    InitWithList {
        list: LineString,
    },
}

impl Operation {
    pub fn apply(&self, polyline: &mut LineString) -> ApplicationResult<()> {
        match self {
            Self::Add { pos, coord } => Ok(polyline.insert(*pos, coord.clone())?),
            Self::Remove { pos, coord } => {
                let ref removed = polyline.remove(*pos)?;
                (coord == removed)
                    .then(|| ())
                    .ok_or(ApplicationError::DomainError(
                        "Contradiction on remove".into(),
                    ))
            }
            Self::Move { pos, from, to } => {
                let ref moved = polyline.replace(*pos, to.clone())?;
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
        match self.clone() {
            Self::Add { pos, coord } => Self::Remove { pos, coord },
            Self::Remove { pos, coord } => Self::Add { pos, coord },
            Self::Move { pos, from, to } => Self::Move {
                pos,
                from: to,
                to: from,
            },
            Self::Clear { org_list: list } => Self::InitWithList { list },
            Self::InitWithList { list } => Self::Clear { org_list: list },
        }
    }
}

/// struct to make initializing enum Operation easier
#[derive(Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct OperationStruct {
    code: String,
    pos: Option<usize>,
    polyline: LineString,
}

impl OperationStruct {
    pub fn new(
        code: String,
        pos: Option<usize>,
        org_coord: Option<Coordinate>,
        new_coord: Option<Coordinate>,
        polyline: Option<LineString>,
    ) -> ApplicationResult<Self> {
        let polyline = if vec!["clear", "init"].contains(&(&code as &str)) {
            polyline.ok_or(ApplicationError::DomainError(format!(
                "Must give polyline for code {}",
                code
            )))?
        } else {
            org_coord
                .clone()
                .or(new_coord.clone())
                .map(|c1| LineString::from(vec![c1, new_coord.or(org_coord).unwrap()]))
                .or(polyline)
                .ok_or(ApplicationError::DomainError(
                    "Must give new_coord or org_coord or polyline for OperationStruct::new".into(),
                ))?
        };
        Ok(Self {
            code,
            pos,
            polyline,
        })
    }
}

impl TryFrom<Operation> for OperationStruct {
    type Error = ApplicationError;

    fn try_from(value: Operation) -> Result<Self, Self::Error> {
        match value {
            Operation::Add { pos, coord } => {
                OperationStruct::new("add".into(), Some(pos), None, Some(coord), None)
            }
            Operation::Remove { pos, coord } => {
                OperationStruct::new("rm".into(), Some(pos), Some(coord), None, None)
            }
            Operation::Move { pos, from, to } => {
                OperationStruct::new("mv".into(), Some(pos), Some(from), Some(to), None)
            }
            Operation::Clear { org_list } => {
                OperationStruct::new("clear".into(), None, None, None, Some(org_list))
            }
            Operation::InitWithList { list } => {
                OperationStruct::new("init".into(), None, None, None, Some(list))
            }
        }
    }
}

impl TryFrom<OperationStruct> for Operation {
    type Error = ApplicationError;

    fn try_from(value: OperationStruct) -> Result<Self, Self::Error> {
        let operation = match &value.code as &str {
            "add" => Operation::Add {
                pos: value.pos.ok_or(ApplicationError::DomainError(
                    "No pos given for Operation::Add".into(),
                ))?,
                coord: value.polyline.get(0)?.clone(),
            },
            "rm" => Operation::Remove {
                pos: value.pos.ok_or(ApplicationError::DomainError(
                    "No pos given for Operation::Remove".into(),
                ))?,
                coord: value.polyline.get(1)?.clone(),
            },
            "mv" => Operation::Move {
                pos: value.pos.ok_or(ApplicationError::DomainError(
                    "No pos given for Operation::Move".into(),
                ))?,
                from: value.polyline.get(0)?.clone(),
                to: value.polyline.get(1)?.clone(),
            },
            "clear" => Operation::Clear {
                org_list: value.polyline,
            },
            "init" => Operation::InitWithList {
                list: value.polyline,
            },
            _ => {
                return Err(ApplicationError::DomainError(format!(
                    "Invalid operation code {}",
                    value.code
                )));
            }
        };
        Ok(operation)
    }
}

pub trait OperationRepository {
    fn find_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<Vec<Operation>>;

    fn update_by_route_id(
        &self,
        route_id: &RouteId,
        op_list: &Vec<Operation>,
    ) -> ApplicationResult<()>;

    fn delete_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<()>;
}
