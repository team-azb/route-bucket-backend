use std::convert::TryFrom;
use std::slice::{Iter, IterMut};

use getset::Getters;
use serde::Serialize;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::types::SegmentId;
use crate::model::{Coordinate, Distance, Polyline};

#[derive(Clone, Debug, Getters, Serialize)]
#[get = "pub"]
pub struct Segment {
    #[serde(skip_serializing)]
    pub(super) id: SegmentId,
    #[serde(skip_serializing)]
    pub(super) start: Coordinate,
    #[serde(skip_serializing)]
    pub(super) goal: Coordinate,
    pub(super) points: Vec<Coordinate>,
}

impl Segment {
    pub fn new_empty(start: Coordinate, goal: Coordinate) -> Self {
        Self {
            id: SegmentId::new(),
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

impl TryFrom<(String, String)> for Segment {
    type Error = ApplicationError;

    fn try_from((id_str, polyline_str): (String, String)) -> ApplicationResult<Self> {
        let err = ApplicationError::DomainError(
            "Cannot Initialize Segment from an empty Vec<Coordinate>!".into(),
        );
        let points = Vec::try_from(Polyline::from(polyline_str))?;
        Ok(Segment {
            id: SegmentId::from_string(id_str),
            start: points.first().ok_or(err.clone())?.clone(),
            goal: points.last().ok_or(err.clone())?.clone(),
            points,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::convert::TryInto;

    use rstest::rstest;

    use crate::model::route::coordinate::tests::CoordinateFixtures;
    use crate::model::Elevation;

    use super::*;

    pub trait SegmentFixtures {
        fn yokohama_to_tokyo(set_ele: bool, set_dist: bool) -> Segment {
            Segment {
                start: Coordinate::yokohama(),
                goal: Coordinate::tokyo(),
                points: Coordinate::yokohama_to_tokyo_coords(set_ele, set_dist),
            }
        }

        fn tokyo_to_chiba(set_ele: bool, set_dist: bool) -> Segment {
            Segment {
                start: Coordinate::tokyo(),
                goal: Coordinate::chiba(),
                points: Coordinate::tokyo_to_chiba_coords(set_ele, set_dist),
            }
        }

        fn yokohama_to_chiba(set_ele: bool, set_dist: bool) -> Segment {
            Segment {
                start: Coordinate::yokohama(),
                goal: Coordinate::chiba(),
                points: Coordinate::yokohama_to_chiba_coords(set_ele, set_dist),
            }
        }

        // The step number corresponds to the execution order of the operations in operation.rs.
        fn yokohama_end(set_ele: bool, set_dist: bool) -> Segment {
            let yokohama = Coordinate::yokohama();
            let mut yokohama_verbose = yokohama.clone();

            yokohama_verbose
                .set_elevation(set_ele.then(|| Elevation::try_from(1).unwrap()))
                .unwrap();
            if set_dist {
                yokohama_verbose.set_distance_from_start(Distance::try_from(0.).unwrap());
            }

            Segment {
                start: yokohama.clone(),
                goal: yokohama,
                points: vec![yokohama_verbose],
            }
        }

        fn yokohama_to_tokyo_end(set_ele: bool, set_dist: bool) -> Segment {
            let tokyo = Coordinate::chiba();
            let mut tokyo_verbose = tokyo.clone();

            tokyo_verbose
                .set_elevation(set_ele.then(|| 4.try_into().unwrap()))
                .unwrap();
            if set_dist {
                tokyo_verbose.set_distance_from_start(29434.629256467866.try_into().unwrap());
            }

            Segment {
                start: tokyo.clone(),
                goal: tokyo,
                points: vec![tokyo_verbose],
            }
        }

        fn yokohama_to_chiba_end(set_ele: bool, set_dist: bool) -> Segment {
            let chiba = Coordinate::chiba();
            let mut chiba_verbose = chiba.clone();

            chiba_verbose
                .set_elevation(set_ele.then(|| 11.try_into().unwrap()))
                .unwrap();
            if set_dist {
                chiba_verbose.set_distance_from_start(61926.0425172123.try_into().unwrap());
            }

            Segment {
                start: chiba.clone(),
                goal: chiba,
                points: vec![chiba_verbose],
            }
        }

        fn yokohama_to_chiba_via_tokyo_end(set_ele: bool, set_dist: bool) -> Segment {
            let chiba = Coordinate::chiba();
            let mut chiba_verbose = chiba.clone();

            chiba_verbose
                .set_elevation(set_ele.then(|| 11.try_into().unwrap()))
                .unwrap();
            if set_dist {
                chiba_verbose.set_distance_from_start(63439.42063598467.try_into().unwrap());
            }

            Segment {
                start: chiba.clone(),
                goal: chiba,
                points: vec![chiba_verbose],
            }
        }
    }

    impl SegmentFixtures for Segment {}

    #[rstest]
    #[case::single_point_segment(Segment::yokohama_end(true, true), 0.)]
    #[case::yokohama_to_tokyo(Segment::yokohama_to_tokyo(true, true), 29434.629256467866)]
    fn return_last_distance_from_start_as_distance(#[case] seg: Segment, #[case] expected: f64) {
        assert_eq!(seg.get_distance().value(), expected)
    }

    #[rstest]
    #[case::yokohama_to_tokyo(
        Segment::new_empty(Coordinate::yokohama(), Coordinate::tokyo()),
        Coordinate::yokohama_to_tokyo_coords(true, true),
        Segment::yokohama_to_tokyo(true, true)
    )]
    fn can_set_points(
        #[case] mut empty_seg: Segment,
        #[case] points: Vec<Coordinate>,
        #[case] expected_segment: Segment,
    ) {
        empty_seg.set_points(points).unwrap();
        assert_eq!(empty_seg, expected_segment)
    }

    #[rstest]
    #[case::yokohama_to_tokyo(Segment::yokohama_to_tokyo(true, true), Segment::tokyo_to_chiba(true, true).points)]
    fn cannot_set_points_twice(#[case] mut filled_seg: Segment, #[case] points: Vec<Coordinate>) {
        assert!(matches!(
            filled_seg.set_points(points),
            Err(ApplicationError::DomainError(_))
        ))
    }

    #[rstest]
    #[case::reset_nothing(Segment::yokohama_to_tokyo(true, true), None, None)]
    #[case::reset_start(
        Segment::yokohama_to_tokyo(true, true),
        Some(Coordinate::tokyo()),
        None
    )]
    #[case::reset_goal(
        Segment::yokohama_to_tokyo(true, true),
        None,
        Some(Coordinate::chiba())
    )]
    #[case::reset_both(
        Segment::yokohama_to_tokyo(true, true),
        Some(Coordinate::tokyo()),
        Some(Coordinate::chiba())
    )]
    fn can_reset_endpoints(
        #[case] mut seg: Segment,
        #[case] start: Option<Coordinate>,
        #[case] goal: Option<Coordinate>,
    ) {
        seg.reset_endpoints(start.clone(), goal.clone());

        let new_start = start.unwrap_or(seg.start.clone());
        let new_goal = goal.unwrap_or(seg.goal.clone());

        assert_eq!(seg, Segment::new_empty(new_start, new_goal))
    }

    #[rstest]
    #[case::reset_nothing(
        Coordinate::yokohama_to_tokyo_coords(false, false),
        Segment::yokohama_to_tokyo(false, false)
    )]
    fn can_be_converted_from_coords(
        #[case] coords: Vec<Coordinate>,
        #[case] expected_seg: Segment,
    ) {
        assert_eq!(Segment::try_from(coords).unwrap(), expected_seg)
    }

    #[rstest]
    #[case::reset_nothing(Coordinate::empty_coords())]
    fn cannot_be_converted_from_empty_coords(#[case] coords: Vec<Coordinate>) {
        assert!(matches!(
            Segment::try_from(coords),
            Err(ApplicationError::DomainError(_))
        ))
    }

    // NOTE: TryFrom<Polyline>, TryFrom<String> は，TryFrom<Vec<Coordinate>>を使ったエイリアスと判断して、
    // テストには含めず
}
