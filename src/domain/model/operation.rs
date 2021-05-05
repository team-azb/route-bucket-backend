use crate::domain::model::polyline::{Coordinate, Polyline};
use crate::domain::model::types::RouteId;
use crate::utils::error::{ApplicationError, ApplicationResult};
use getset::Getters;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        org_list: Polyline,
    },
    // reverse operation for Clear
    InitWithList {
        list: Polyline,
    },
}

impl Operation {
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

#[derive(Getters)]
#[get = "pub"]
pub struct OperationStruct {
    code: String,
    pos: Option<usize>,
    polyline: Polyline,
}

impl OperationStruct {
    pub fn new(
        code: String,
        pos: Option<usize>,
        org_coord: Option<Coordinate>,
        new_coord: Option<Coordinate>,
        polyline: Option<Polyline>,
    ) -> ApplicationResult<Self> {
        let polyline = if vec!["clear", "init"].contains(&(&code as &str)) {
            polyline.ok_or(ApplicationError::DomainError(format!(
                "Must give polyline for code {}",
                code
            )))?
        } else {
            org_coord.map_or(
                polyline.ok_or(ApplicationError::DomainError(
                    "Must give new_coord or org_coord or polyline for OperationStruct::new".into(),
                ))?,
                |c1| {
                    new_coord
                        .map_or(vec![c1.clone()], |c2| vec![c1.clone(), c2.clone()])
                        .into()
                },
            )
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
                coord: value.polyline.get(0)?.clone(),
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
