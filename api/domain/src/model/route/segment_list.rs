use std::cmp::max;
use std::convert::TryInto;
use std::slice::{Iter, IterMut};

use geo::algorithm::haversine_distance::HaversineDistance;
use getset::Getters;
use gpx::{Track, TrackSegment, Waypoint};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::Serialize;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::{Distance, Elevation};

use super::coordinate::Coordinate;

pub use self::segment::Segment;

mod segment;

#[derive(Clone, Debug, Serialize, Getters)]
#[get = "pub"]
pub struct SegmentList {
    segments: Vec<Segment>,
}

impl SegmentList {
    pub fn get_total_distance(&self) -> ApplicationResult<Distance> {
        if let Some(last_seg) = self.segments.last() {
            let last_point =
                last_seg
                    .points
                    .last()
                    .ok_or(ApplicationError::DomainError(format!(
                        "Last segment cannot be empty at get_total_distance! ({:?})",
                        last_seg
                    )))?;
            last_point
                .distance_from_start()
                .clone()
                .ok_or(ApplicationError::DomainError(
                    format!("Failed to calculate total distance. {:?}", self).into(),
                ))
        } else {
            Ok(Distance::zero())
        }
    }

    pub fn calc_elevation_gain(&self) -> Elevation {
        self.iter()
            .par_bridge()
            .fold(Elevation::zero, |total_gain, seg| {
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
                total_gain + gain
            })
            .sum::<Elevation>()
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

    pub fn insert_waypoint(&mut self, pos: usize, coord: Coordinate) -> ApplicationResult<()> {
        let org_len = self.segments.len();
        if pos <= org_len {
            if pos == 0 {
                let goal = self
                    .segments
                    .first()
                    .map(|seg| seg.start.clone())
                    .unwrap_or(coord.clone());
                self.segments.insert(0, Segment::new_empty(coord, goal));
            } else {
                let org_seg = self.segments.remove(pos - 1);
                let start = org_seg.start.clone();
                let goal = if pos == org_len {
                    coord.clone()
                } else {
                    org_seg.goal
                };
                self.segments
                    .insert(pos - 1, Segment::new_empty(start, coord.clone()));
                self.segments.insert(pos, Segment::new_empty(coord, goal));
            }
            Ok(())
        } else {
            Err(ApplicationError::DomainError(format!(
                "pos({}) cannot be greater than segments.len()({}) at SegmentList::insert_waypoint",
                pos,
                self.segments.len()
            )))
        }
    }

    pub fn remove_waypoint(&mut self, pos: usize) -> ApplicationResult<()> {
        let org_len = self.segments.len();
        if org_len == 0 {
            return Err(ApplicationError::DomainError(
                "segments.len() cannot be 0 at SegmentList::remove_waypoint".into(),
            ));
        }

        if pos < org_len {
            let org_second_seg = self.segments.remove(pos);
            if pos > 0 {
                let org_first_seg = self.segments.remove(pos - 1);
                let start = org_first_seg.start.clone();
                let goal = if pos == org_len - 1 {
                    org_first_seg.start
                } else {
                    org_second_seg.goal
                };
                self.segments
                    .insert(pos - 1, Segment::new_empty(start, goal));
            }
            Ok(())
        } else {
            Err(ApplicationError::DomainError(format!(
                "pos({}) cannot be greater than or equal to segments.len()({}) at SegmentList::remove_waypoint",
                pos,
                self.segments.len()
            )))
        }
    }

    pub fn move_waypoint(&mut self, pos: usize, coord: Coordinate) -> ApplicationResult<()> {
        self.remove_waypoint(pos)?;
        self.insert_waypoint(pos, coord)?;

        Ok(())
    }

    pub fn gather_waypoints(&self) -> Vec<Coordinate> {
        self.segments.iter().map(|seg| seg.start.clone()).collect()
    }

    pub fn into_segments_in_between(self) -> Vec<Segment> {
        let mut segments: Vec<Segment> = self.into();
        if segments.len() > 0 {
            segments.pop();
        }
        segments
    }

    pub fn iter(&self) -> Iter<Segment> {
        self.segments.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Segment> {
        self.segments.iter_mut()
    }
}

impl From<Vec<Segment>> for SegmentList {
    fn from(segments: Vec<Segment>) -> Self {
        Self { segments }
    }
}

impl From<SegmentList> for Vec<Segment> {
    fn from(seg_list: SegmentList) -> Self {
        seg_list.segments
    }
}

impl From<SegmentList> for Track {
    fn from(seg_list: SegmentList) -> Self {
        let mut trk = Self::new();
        trk.segments.push(TrackSegment::new());
        trk.segments[0].points = seg_list
            .segments
            .into_iter()
            .map(|seg| seg.points)
            .concat()
            .into_iter()
            .map(Waypoint::from)
            .collect_vec();
        trk
    }
}
