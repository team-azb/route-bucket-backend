use std::convert::TryFrom;
use std::mem::swap;

use getset::Getters;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::coordinate::Coordinate;
use crate::model::segment::SegmentList;

#[derive(Clone, Debug)]
pub enum OperationType {
    Add,
    Remove,
    Move,
    Clear,
    InitWithList, // reverse operation for Clear
}

impl OperationType {
    pub fn reverse(&self) -> Self {
        match *self {
            OperationType::Add => OperationType::Remove,
            OperationType::Remove => OperationType::Add,
            OperationType::Move => OperationType::Move,
            OperationType::Clear => OperationType::InitWithList,
            OperationType::InitWithList => OperationType::Clear,
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
            "cl" => Ok(OperationType::Clear),
            "in" => Ok(OperationType::InitWithList),
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
            OperationType::Clear => "cl",
            OperationType::InitWithList => "in",
        }
        .into()
    }
}

#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct Operation {
    op_type: OperationType,
    start_pos: usize,
    org_coords: Vec<Coordinate>,
    new_coords: Vec<Coordinate>,
}

impl Operation {
    pub fn new(
        op_type: OperationType,
        start_pos: usize,
        org_coords: Vec<Coordinate>,
        new_coords: Vec<Coordinate>,
    ) -> Self {
        Self {
            op_type,
            start_pos,
            org_coords,
            new_coords,
        }
    }

    pub fn new_add(pos: usize, coord: Coordinate) -> Self {
        Self::new(OperationType::Add, pos, Vec::new(), vec![coord])
    }

    pub fn new_remove(pos: usize, org_waypoints: Vec<Coordinate>) -> Self {
        let org = org_waypoints[pos].clone();
        Self::new(OperationType::Remove, pos, vec![org], Vec::new())
    }

    pub fn new_move(pos: usize, coord: Coordinate, org_waypoints: Vec<Coordinate>) -> Self {
        let org = org_waypoints[pos].clone();
        Self::new(OperationType::Move, pos, vec![org], vec![coord])
    }

    pub fn new_clear(org_waypoints: Vec<Coordinate>) -> Self {
        Self::new(OperationType::Clear, 0, org_waypoints, Vec::new())
    }

    pub fn apply(&self, seg_list: &mut SegmentList) -> ApplicationResult<()> {
        seg_list.replace_range(
            self.start_pos..self.start_pos + self.org_coords.len(),
            self.new_coords.clone(),
        )
    }

    pub fn reverse(&mut self) {
        self.op_type = self.op_type.reverse();
        swap(&mut self.org_coords, &mut self.new_coords);
    }
}
