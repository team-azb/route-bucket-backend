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
            seg.iter_mut()
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
                *offset += seg.get_distance();
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

impl From<SegmentList> for (Vec<Coordinate>, Vec<Segment>) {
    fn from(seg_list: SegmentList) -> Self {
        let (coords, segments) = seg_list
            .0
            .into_iter()
            .enumerate()
            .map(|(i, seg)| {
                let coords = if i == 0 {
                    vec![seg.start, seg.goal]
                } else {
                    vec![seg.goal]
                };
                (coords, seg.points)
            })
            .unzip();
        (coords.into_iter().concat().collect_vec(), segments)
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
    #[serde(skip_serializing)]
    start: Coordinate,
    #[serde(skip_serializing)]
    goal: Coordinate,
    points: Vec<Coordinate>,
}

impl Segment {
    pub fn get_distance(&self) -> Distance {
        self.points
            .last()
            .map(Coordinate::distance_from_start)
            .flatten()
            .unwrap_or(Distance::zero())
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn iter(&self) -> Iter<Coordinate> {
        self.points.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Coordinate> {
        self.points.iter_mut()
    }
}

impl TryFrom<Vec<Coordinate>> for Segment {
    type Error = ApplicationError;

    fn try_from(points: Vec<Coordinate>) -> ApplicationResult<Self> {
        let err = ApplicationError::DomainError(
            "Cannot Initialize Segment from an empty Vec<Coordinate>!".into(),
        );
        Ok(Segment {
            start: points.first().ok_or(&err)?.clone(),
            goal: points.last().ok_or(&err)?.clone(),
            points,
        })
    }
}

impl TryFrom<Polyline> for Segment {
    type Error = ApplicationError;

    fn try_from(polyline: Polyline) -> ApplicationResult<Self> {
        Segment::try_from(Vec::try_from(polyline)?)
    }
}

impl TryFrom<String> for Segment {
    type Error = ApplicationError;

    fn try_from(polyline_str: String) -> ApplicationResult<Self> {
        Segment::try_from(Polyline::from(polyline_str))
    }
}
