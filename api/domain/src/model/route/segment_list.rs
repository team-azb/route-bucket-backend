use std::cmp::max;
use std::convert::TryInto;
use std::ops::Range;
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
    // TODO: この二つをなくす
    removed_range: Option<Range<usize>>,
    inserted_range: Option<Range<usize>>,
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
        if pos <= self.segments.len() {
            if pos == 0 {
                let goal = self
                    .segments
                    .first()
                    .map(|seg| seg.start.clone())
                    .unwrap_or(coord.clone());
                self.segments.insert(0, Segment::new_empty(coord, goal));
                self.inserted_range.insert(0..1);
            } else {
                let goal = if pos == self.segments.len() {
                    coord.clone()
                } else {
                    self.segments[pos - 1].goal.clone()
                };
                self.segments[pos - 1].reset_endpoints(None, Some(coord.clone()));
                self.segments.insert(pos, Segment::new_empty(coord, goal));
                self.inserted_range.insert(pos - 1..pos + 1);
                self.removed_range.insert(pos - 1..pos);
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
        if self.segments.len() == 0 {
            return Err(ApplicationError::DomainError(
                "segments.len() cannot be 0 at SegmentList::remove_waypoint".into(),
            ));
        }

        if pos <= self.segments.len() {
            let removed_seg = self.segments.remove(pos);
            if pos > 0 {
                let goal = if pos == self.segments.len() {
                    self.segments[pos - 1].start.clone()
                } else {
                    removed_seg.goal
                };
                self.segments[pos - 1].reset_endpoints(None, Some(goal));
                self.inserted_range.insert(pos - 1..pos);
                self.removed_range.insert(pos - 1..pos + 1);
            } else {
                self.inserted_range.insert(pos..pos);
                self.removed_range.insert(pos..pos + 1);
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

        let range_start = pos.checked_sub(1).unwrap_or(0);
        let range_end = pos.checked_add(1).unwrap_or(self.segments.len());
        self.inserted_range.insert(range_start..range_end);
        self.removed_range.insert(range_start..range_end);

        Ok(())
    }

    pub fn get_inserted_slice(&self) -> ApplicationResult<&[Segment]> {
        if let Some(inserted_range) = self.inserted_range.clone() {
            Ok(&self.segments[inserted_range])
        } else {
            Err(ApplicationError::DomainError(
                "SegmentList still hasn't been inserted at get_inserted_slice".into(),
            ))
        }
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
        Self {
            segments,
            removed_range: None,
            inserted_range: None,
        }
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
