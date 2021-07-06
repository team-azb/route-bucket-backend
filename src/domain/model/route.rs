use derive_more::From;
use getset::{Getters, MutGetters};
use serde::{Deserialize, Serialize};

use crate::domain::model::coordinate::Coordinate;
use crate::domain::model::operation::Operation;
use crate::domain::model::segment::SegmentList;
use crate::domain::model::types::{Elevation, RouteId};
use crate::utils::error::{ApplicationError, ApplicationResult};

#[derive(Debug, From, Getters, MutGetters)]
#[get = "pub"]
pub struct Route {
    info: RouteInfo,
    op_list: Vec<Operation>,
    #[getset(get_mut = "pub")]
    seg_list: SegmentList,
}

impl Route {
    pub fn new(info: RouteInfo, op_list: Vec<Operation>, seg_list: SegmentList) -> Self {
        Self {
            info,
            op_list,
            seg_list,
        }
    }

    pub fn get_operation(&self, pos: usize) -> ApplicationResult<&Operation> {
        self.op_list
            .get(pos)
            .ok_or(ApplicationError::DomainError(format!(
                "Index {} out of range for RouteEditor.op_list!(len={})",
                pos,
                self.op_list.len()
            )))
    }

    pub fn push_operation(&mut self, op: Operation) -> ApplicationResult<()> {
        // pos以降の要素は全て捨てる
        self.op_list.truncate(self.info.op_num);
        self.op_list.push(op);

        self.apply_operation(false)
    }

    pub fn redo_operation(&mut self) -> ApplicationResult<()> {
        if self.info.op_num < self.op_list.len() {
            self.apply_operation(false)
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to redo.",
            ))
        }
    }

    pub fn undo_operation(&mut self) -> ApplicationResult<()> {
        if self.info.op_num > 0 {
            self.apply_operation(true)
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to undo.",
            ))
        }
    }

    pub fn calc_elevation_gain(&self) -> ApplicationResult<Elevation> {
        self.seg_list.calc_elevation_gain()
    }

    pub fn gather_waypoints(&self) -> Vec<Coordinate> {
        self.seg_list.gather_waypoints()
    }

    fn apply_operation(&mut self, reverse: bool) -> ApplicationResult<()> {
        let mut op;
        if reverse {
            self.info.op_num -= 1;
            op = self.get_operation(self.info.op_num)?.clone();
            op.reverse()
        } else {
            op = self.get_operation(self.info.op_num)?.clone();
            self.info.op_num += 1;
        };

        op.apply(&mut self.seg_list)?;

        Ok(())
    }
}

#[derive(Debug, Getters, Deserialize, Serialize)]
#[get = "pub"]
pub struct RouteInfo {
    id: RouteId,
    name: String,
    #[serde(skip_serializing)]
    op_num: usize,
}

impl RouteInfo {
    pub fn new(id: RouteId, name: &String, op_num: usize) -> RouteInfo {
        RouteInfo {
            id,
            name: name.clone(),
            op_num,
        }
    }

    pub fn rename(&mut self, name: &String) {
        self.name = name.clone();
    }
}
