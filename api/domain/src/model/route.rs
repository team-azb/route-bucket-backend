use std::slice::IterMut;

use derive_more::{From, Into};
use getset::Getters;
use gpx::{Gpx, GpxVersion};

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::types::Elevation;
use crate::model::Distance;

use self::coordinate::Coordinate;
use self::operation::Operation;
use self::route_info::RouteInfo;
use self::segment_list::{Segment, SegmentList};

pub(crate) mod coordinate;
pub(crate) mod operation;
pub(crate) mod route_gpx;
pub(crate) mod route_info;
pub(crate) mod segment_list;

#[derive(Debug, From, Into, Getters)]
#[get = "pub"]
pub struct Route {
    info: RouteInfo,
    op_list: Vec<Operation>,
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

    // methods from SegmentList

    pub fn calc_elevation_gain(&self) -> Elevation {
        self.seg_list.calc_elevation_gain()
    }

    pub fn attach_distance_from_start(&mut self) -> ApplicationResult<()> {
        self.seg_list.attach_distance_from_start()
    }

    pub fn get_total_distance(&self) -> ApplicationResult<Distance> {
        self.seg_list.get_total_distance()
    }

    pub fn gather_waypoints(&self) -> Vec<Coordinate> {
        self.seg_list.gather_waypoints()
    }

    pub fn iter_seg_mut(&mut self) -> IterMut<Segment> {
        self.seg_list.iter_mut()
    }

    pub fn into_segments_in_between(self) -> Vec<Segment> {
        self.seg_list.into_segments_in_between()
    }
}

impl From<Route> for Gpx {
    fn from(route: Route) -> Self {
        Gpx {
            version: GpxVersion::Gpx11,
            metadata: Some(route.info.into()),
            tracks: vec![route.seg_list.into()],
            ..Default::default()
        }
    }
}
