use std::convert::TryFrom;
use std::slice::{Iter, IterMut};

use getset::Getters;
use serde::Serialize;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::{Coordinate, Distance, Polyline};

#[derive(Clone, Debug, Getters, Serialize)]
#[get = "pub"]
pub struct Segment {
    #[serde(skip_serializing)]
    pub(super) start: Coordinate,
    #[serde(skip_serializing)]
    pub(super) goal: Coordinate,
    pub(super) points: Vec<Coordinate>,
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
