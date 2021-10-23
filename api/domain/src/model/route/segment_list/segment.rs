use std::convert::TryFrom;
use std::slice::{Iter, IterMut};

use derive_more::IntoIterator;
use getset::Getters;
use serde::Serialize;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::types::SegmentId;
use crate::model::{Coordinate, Distance, Polyline};

#[cfg(any(test, feature = "fixtures"))]
use derivative::Derivative;

#[derive(Clone, Debug, Getters, Serialize, IntoIterator)]
#[get = "pub"]
#[cfg_attr(any(test, feature = "fixtures"), derive(Derivative))]
#[cfg_attr(any(test, feature = "fixtures"), derivative(PartialEq))]
pub struct Segment {
    #[serde(skip_serializing)]
    #[cfg_attr(any(test, feature = "fixtures"), derivative(PartialEq = "ignore"))]
    pub(super) id: SegmentId,
    #[serde(skip_serializing)]
    pub(super) start: Coordinate,
    #[serde(skip_serializing)]
    pub(super) goal: Coordinate,
    #[into_iterator(owned)]
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
            .map(|coord| *coord.distance_from_start())
            .flatten()
            .unwrap_or_else(Distance::zero)
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
            start: points.first().ok_or_else(|| err.clone())?.clone(),
            goal: points.last().ok_or(err)?.clone(),
            points,
        })
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod tests {
    use rstest::{fixture, rstest};

    use crate::model::route::coordinate::tests::CoordinateFixtures;

    use super::*;

    #[fixture]
    fn yokohama_to_tokyo() -> Segment {
        Segment::yokohama_to_tokyo(true, Some(0.), false)
    }

    #[fixture]
    fn yokohama_to_tokyo_empty() -> Segment {
        Segment::yokohama_to_tokyo(false, None, true)
    }

    #[fixture]
    fn yokohama_to_tokyo_coords() -> Vec<Coordinate> {
        Coordinate::yokohama_to_tokyo_coords(true, Some(0.))
    }

    #[rstest]
    #[case::empty_segment(yokohama_to_tokyo_empty(), 0.)]
    #[case::yokohama_to_tokyo(yokohama_to_tokyo(), 26936.42633640023)]
    fn can_return_last_distance_from_start_as_distance(
        #[case] seg: Segment,
        #[case] expected: f64,
    ) {
        assert_eq!(seg.get_distance().value(), expected)
    }

    #[rstest]
    fn can_set_points(
        #[from(yokohama_to_tokyo_empty)] mut empty_seg: Segment,
        #[from(yokohama_to_tokyo_coords)] points: Vec<Coordinate>,
        #[from(yokohama_to_tokyo)] expected_segment: Segment,
    ) {
        empty_seg.set_points(points).unwrap();
        assert_eq!(&empty_seg, &expected_segment)
    }

    #[rstest]
    fn cannot_set_points_twice(
        #[from(yokohama_to_tokyo)] mut filled_seg: Segment,
        #[from(yokohama_to_tokyo_coords)] points: Vec<Coordinate>,
    ) {
        assert!(matches!(
            filled_seg.set_points(points),
            Err(ApplicationError::DomainError(_))
        ))
    }

    #[rstest]
    #[case("yokohama-to-tokyo____".into(), "{inwE}uesYcoh@u|Z".into(), Segment::yokohama_to_tokyo(false, None, false))]
    fn pair_of_string_can_be_converted_to_segment(
        #[case] id: String,
        #[case] polyline: String,
        #[case] expected_seg: Segment,
    ) {
        let convert_result = Segment::try_from((id.clone(), polyline));
        assert_eq!(convert_result, Ok(expected_seg));
        assert_eq!(convert_result.unwrap().id.to_string(), id);
    }

    macro_rules! point_segment {
        ($fix_name:ident, $set_ele:expr, $dist_offset:expr, $init_empty:expr) => {
            Segment {
                id: SegmentId::from_string(format!("{:_<21}", stringify!($fix_name))),
                start: Coordinate::$fix_name(false, None),
                goal: Coordinate::$fix_name(false, None),
                points: if $init_empty {
                    Vec::new()
                } else {
                    vec![Coordinate::$fix_name($set_ele, $dist_offset)]
                },
            }
        };
    }

    pub trait SegmentFixtures {
        fn yokohama_to_tokyo(set_ele: bool, dist_offset: Option<f64>, init_empty: bool) -> Segment {
            Segment {
                id: SegmentId::from_string("yokohama-to-tokyo____".into()),
                start: Coordinate::yokohama(false, None),
                goal: Coordinate::tokyo(false, None),
                points: if init_empty {
                    Vec::new()
                } else {
                    Coordinate::yokohama_to_tokyo_coords(set_ele, dist_offset)
                },
            }
        }

        fn tokyo_to_chiba(set_ele: bool, dist_offset: Option<f64>, init_empty: bool) -> Segment {
            Segment {
                id: SegmentId::from_string("tokyo-to-chiba_______".into()),
                start: Coordinate::tokyo(false, None),
                goal: Coordinate::chiba(false, None),
                points: if init_empty {
                    Vec::new()
                } else {
                    Coordinate::tokyo_to_chiba_coords(set_ele, dist_offset)
                },
            }
        }

        fn yokohama_to_chiba(set_ele: bool, dist_offset: Option<f64>, init_empty: bool) -> Segment {
            Segment {
                id: SegmentId::from_string("yokohama-to-chiba____".into()),
                start: Coordinate::yokohama(false, None),
                goal: Coordinate::chiba(false, None),
                points: if init_empty {
                    Vec::new()
                } else {
                    Coordinate::yokohama_to_chiba_coords(set_ele, dist_offset)
                },
            }
        }

        fn yokohama(set_ele: bool, dist_offset: Option<f64>, init_empty: bool) -> Segment {
            point_segment!(yokohama, set_ele, dist_offset, init_empty)
        }

        fn tokyo(set_ele: bool, dist_offset: Option<f64>, init_empty: bool) -> Segment {
            point_segment!(tokyo, set_ele, dist_offset, init_empty)
        }

        fn chiba(set_ele: bool, dist_offset: Option<f64>, init_empty: bool) -> Segment {
            point_segment!(chiba, set_ele, dist_offset, init_empty)
        }
    }

    impl SegmentFixtures for Segment {}
}
