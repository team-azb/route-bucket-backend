use std::cmp::max;
use std::convert::{TryFrom, TryInto};
use std::ops::Range;
use std::slice::{Iter, IterMut};

use geo::algorithm::haversine_distance::HaversineDistance;
use getset::Getters;
use gpx::{Track, TrackSegment, Waypoint};
use itertools::Itertools;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::Serialize;

use crate::domain::model::coordinate::Coordinate;
use crate::domain::model::types::{Distance, Elevation, Polyline};
use crate::utils::error::{ApplicationError, ApplicationResult};

#[derive(Clone, Debug, Serialize, Getters)]
#[get = "pub"]
pub struct SegmentList {
    segments: Vec<Segment>,
    replaced_range: Range<usize>,
}

impl SegmentList {
    pub fn get_total_distance(&self) -> ApplicationResult<Distance> {
        self.segments
            .last()
            .map(|seg| {
                seg.points
                    .last()
                    .map(|coord| coord.distance_from_start().clone())
                    .flatten()
            })
            .flatten()
            .ok_or(ApplicationError::DomainError(
                "Failed to calculate total distance.".into(),
            ))
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

        self.segments.splice(range.clone(), replace_with);

        self.replaced_range = range;
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
        Self {
            segments,
            replaced_range: (0..0),
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

#[derive(Clone, Debug, Getters, Serialize)]
#[get = "pub"]
pub struct Segment {
    #[serde(skip_serializing)]
    start: Coordinate,
    #[serde(skip_serializing)]
    goal: Coordinate,
    points: Vec<Coordinate>,
}

impl Segment {
    pub fn new_empty(start: Coordinate, goal: Coordinate) -> Self {
        Self {
            start,
            goal,
            points: Vec::new(),
        }
    }

    pub fn get_distance(&self) -> Distance {
        self.points
            .last()
            .map(|coord| coord.distance_from_start().clone())
            .flatten()
            .unwrap_or(Distance::zero())
    }

    pub fn set_points(&mut self, points: Vec<Coordinate>) -> ApplicationResult<()> {
        if self.is_empty() {
            self.points = points;
            Ok(())
        } else {
            Err(ApplicationError::DomainError(
                "Cannot set_points to a Segment which isn't empty!".into(),
            ))
        }
    }

    pub fn reset_endpoints(&mut self, start_op: Option<Coordinate>, goal_op: Option<Coordinate>) {
        self.start = start_op.unwrap_or(self.start.clone());
        self.goal = goal_op.unwrap_or(self.goal.clone());

        self.points.clear();
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
            start: points.first().ok_or(err.clone())?.clone(),
            goal: points.last().ok_or(err.clone())?.clone(),
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
