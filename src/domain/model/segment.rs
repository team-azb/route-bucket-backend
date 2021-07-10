use std::cmp::max;
use std::convert::{TryFrom, TryInto};
use std::slice::{Iter, IterMut};

use geo::algorithm::haversine_distance::HaversineDistance;
use getset::Getters;
use num_traits::Zero;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::Serialize;

use crate::domain::model::linestring::{Coordinate, LineString};
use crate::domain::model::types::{Distance, Elevation, Polyline};
use crate::utils::error::{ApplicationError, ApplicationResult};

#[derive(Debug, Serialize)]
pub struct SegmentList(Vec<Segment>);

impl SegmentList {
    pub fn calc_elevation_gain(&self) -> ApplicationResult<Elevation> {
        self.iter()
            .par_bridge()
            .fold(i32::zero, |total_gain, seg| {
                let mut gain = Elevation::zero();
                let mut prev_elev = Elevation::max();
                seg.iter().for_each(|coord| {
                    if let Some(elev) = coord.elevation() {
                        gain += max(*elev - prev_elev, 0.try_into().unwrap());
                        prev_elev = *elev;
                    }
                });
                // NOTE: const genericsのあるNumericValueObjectに、Sumがderiveできないので、i32にしている
                // pull request -> https://github.com/JelteF/derive_more/pull/167
                total_gain + gain.value()
            })
            .sum::<i32>()
            .try_into()
    }

    pub fn attach_distance_from_start(&mut self) -> ApplicationResult<()> {
        // compute cumulative distance within the segments
        self.iter_mut().par_bridge().for_each(|seg| {
            seg.distance = seg
                .iter_mut()
                .scan((Distance::zero(), None), |(dist, prev_op), coord| {
                    if let Some(prev_coord) = prev_op {
                        *dist += coord.haversine_distance(prev_coord);
                    }
                    coord.set_distance_from_start(*dist);
                    prev_op.insert(coord.clone());
                    Some(*dist)
                })
                .last()
                .unwrap();
        });

        // convert to global cumulative distance
        self.iter_mut()
            .scan(Distance::zero(), |offset, seg| {
                let prev_offset = *offset;
                *offset += seg.distance;
                Some((seg, prev_offset))
            })
            .par_bridge()
            .for_each(|(seg, offset)| {
                seg.iter_mut().par_bridge().for_each(|coord| {
                    let org_dist = coord.distance_from_start().unwrap();
                    coord.set_distance_from_start(org_dist + offset);
                });
            });

        Ok(())
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
