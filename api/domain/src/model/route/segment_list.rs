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

    pub fn replace_range(
        &mut self,
        mut range: Range<usize>,
        mut waypoints: Vec<Coordinate>,
    ) -> ApplicationResult<()> {
        if self.removed_range.is_some() || self.inserted_range.is_some() {
            return Err(ApplicationError::DomainError(
                "SegmentList has already been modified".into(),
            ));
        }
        if range.start > self.segments.len() {
            return Err(ApplicationError::DomainError(
                "Invalid Range at replace_range!".into(),
            ));
        }

        if range.start > 0 {
            range.start -= 1;
            waypoints.insert(0, self.segments[range.start].start.clone())
        }

        if range.end < self.segments.len() {
            waypoints.push(self.segments[range.end].start.clone())
        } else if let Some(last_ref) = waypoints.last() {
            let last = last_ref.clone();
            waypoints.push(last)
        }

        let replace_with = waypoints
            .into_iter()
            .tuple_windows()
            .map(|(start, goal)| Segment::new_empty(start, goal))
            .collect_vec();

        self.removed_range.insert(range.clone());
        self.inserted_range
            .insert(range.start..(range.start + replace_with.len()));

        self.segments.splice(range.clone(), replace_with);
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
