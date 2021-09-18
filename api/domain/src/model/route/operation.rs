use std::convert::TryFrom;
use std::mem::swap;

use getset::Getters;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use super::coordinate::Coordinate;
use super::segment_list::SegmentList;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationType {
    Add,
    Remove,
    Move,
}

impl OperationType {
    pub fn reverse(&self) -> Self {
        match *self {
            OperationType::Add => OperationType::Remove,
            OperationType::Remove => OperationType::Add,
            OperationType::Move => OperationType::Move,
        }
    }
}

impl TryFrom<String> for OperationType {
    type Error = ApplicationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "ad" => Ok(OperationType::Add),
            "rm" => Ok(OperationType::Remove),
            "mv" => Ok(OperationType::Move),
            _ => Err(ApplicationError::DomainError(format!(
                "Cannot convert {} into OperationType!",
                value
            ))),
        }
    }
}

impl From<OperationType> for String {
    fn from(value: OperationType) -> Self {
        match value {
            OperationType::Add => "ad",
            OperationType::Remove => "rm",
            OperationType::Move => "mv",
        }
        .into()
    }
}

#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct Operation {
    op_type: OperationType,
    pos: usize,
    org_coord: Option<Coordinate>,
    new_coord: Option<Coordinate>,
}

impl Operation {
    pub fn new(
        op_type: OperationType,
        pos: usize,
        org_coord: Option<Coordinate>,
        new_coord: Option<Coordinate>,
    ) -> Self {
        Self {
            op_type,
            pos,
            org_coord,
            new_coord,
        }
    }

    pub fn new_add(pos: usize, coord: Coordinate) -> Self {
        Self::new(OperationType::Add, pos, None, Some(coord))
    }

    pub fn new_remove(pos: usize, org_waypoints: Vec<Coordinate>) -> Self {
        let org = org_waypoints[pos].clone();
        Self::new(OperationType::Remove, pos, Some(org), None)
    }

    pub fn new_move(pos: usize, coord: Coordinate, org_waypoints: Vec<Coordinate>) -> Self {
        let org = org_waypoints[pos].clone();
        Self::new(OperationType::Move, pos, Some(org), Some(coord))
    }

    pub fn apply(&self, seg_list: &mut SegmentList) -> ApplicationResult<()> {
        match self.op_type {
            OperationType::Remove => seg_list.remove_waypoint(self.pos),
            OperationType::Add | OperationType::Move => {
                if let Some(new_coord) = self.new_coord.clone() {
                    if self.op_type == OperationType::Add {
                        seg_list.insert_waypoint(self.pos, new_coord)
                    } else {
                        seg_list.move_waypoint(self.pos, new_coord)
                    }
                } else {
                    Err(ApplicationError::DomainError(
                        "OperationType::{Add | Move} must have new_coord!".into(),
                    ))
                }
            }
        }
    }

    pub fn reverse(&mut self) {
        self.op_type = self.op_type.reverse();
        swap(&mut self.org_coord, &mut self.new_coord);
    }
}
