use std::cmp::max;
use std::convert::{TryFrom, TryInto};
use std::slice::{Iter, IterMut};

use getset::Getters;
use serde::Serialize;

use crate::domain::model::linestring::{Coordinate, LineString};
use crate::domain::model::types::{Distance, Elevation, Polyline};
use crate::utils::error::{ApplicationError, ApplicationResult};

#[derive(Debug, Serialize)]
pub struct SegmentList(Vec<Segment>);

impl SegmentList {
    pub fn calc_elevation_gain(&self) -> ApplicationResult<Elevation> {
        let mut gain = 0.try_into().unwrap();
        let mut prev_elev = Elevation::max();

        self.iter().for_each(|seg| {
            seg.iter().for_each(|coord| {
                if let Some(elev) = coord.elevation() {
                    gain += max(*elev - prev_elev, 0.try_into().unwrap());
                    prev_elev = *elev;
                }
            })
        });

        Ok(gain)
    }

    pub fn iter(&self) -> Iter<Segment> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Segment> {
        self.0.iter_mut()
    }
}

impl From<Vec<Segment>> for SegmentList {
    fn from(seg_vec: Vec<Segment>) -> Self {
        Self(seg_vec)
    }
}

impl From<SegmentList> for Vec<Segment> {
    fn from(seg_list: SegmentList) -> Self {
        seg_list.0
    }
}

#[derive(Debug, Getters, Serialize)]
#[get = "pub"]
pub struct Segment {
    points: LineString,
    distance: Distance,
}

impl Segment {
    pub fn iter(&self) -> Iter<Coordinate> {
        self.points.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Coordinate> {
        self.points.iter_mut()
    }
}

// TODO: 各地点の累計距離の計算は、Coordinateに distance_from_start: Option<f64> みたいなんを追加して、
//     : Segment.calc_distanceみたいなので計算すれば良さそう

impl From<(LineString, Distance)> for Segment {
    fn from((points, distance): (LineString, Distance)) -> Self {
        Segment { points, distance }
    }
}

impl TryFrom<(Polyline, Distance)> for Segment {
    type Error = ApplicationError;

    fn try_from((polyline, distance): (Polyline, Distance)) -> ApplicationResult<Self> {
        Ok(Segment::from((LineString::try_from(polyline)?, distance)))
    }
}
