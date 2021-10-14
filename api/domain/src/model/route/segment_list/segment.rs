use std::convert::TryFrom;
use std::slice::{Iter, IterMut};
use std::str::FromStr;

use derive_more::IntoIterator;
use getset::Getters;
use serde::{Deserialize, Serialize};

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::types::SegmentId;
use crate::model::{Coordinate, Distance, Polyline};

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum DrawingMode {
    FollowRoad,
    Freehand,
}

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
    #[serde(skip_serializing)]
    pub(super) mode: DrawingMode,
    #[into_iterator(owned)]
    pub(super) points: Vec<Coordinate>,
}

impl Segment {
    pub fn new_empty(start: Coordinate, goal: Coordinate, mode: DrawingMode) -> Self {
        Self {
            id: SegmentId::new(),
            start,
            goal,
            mode,
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

impl TryFrom<(String, String, String)> for Segment {
    type Error = ApplicationError;

    fn try_from(
        (id_str, mode_str, polyline_str): (String, String, String),
    ) -> ApplicationResult<Self> {
        let empty_err = ApplicationError::DomainError(
            "Cannot Initialize Segment from an empty Vec<Coordinate>!".into(),
        );
        let points = Vec::try_from(Polyline::from(polyline_str))?;
        Ok(Segment {
            id: SegmentId::from_string(id_str),
            start: points.first().ok_or_else(|| empty_err.clone())?.clone(),
            goal: points.last().ok_or(empty_err)?.clone(),
            mode: DrawingMode::from_str(&mode_str).map_err(|_| {
                ApplicationError::DomainError(format!("Invalid mode: {}", mode_str))
            })?,
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
        Segment::yokohama_to_tokyo(true, Some(0.), false, DrawingMode::FollowRoad)
    }

    #[fixture]
    fn yokohama_to_tokyo_empty() -> Segment {
        Segment::yokohama_to_tokyo(false, None, true, DrawingMode::FollowRoad)
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
    #[case::follow_road("follow_road", DrawingMode::FollowRoad)]
    #[case::freehand("freehand", DrawingMode::Freehand)]
    fn valid_str_can_be_converted_to_drawing_mode(
        #[case] valid_str: &str,
        #[case] expected_mode: DrawingMode,
    ) {
        assert_eq!(DrawingMode::from_str(valid_str), Ok(expected_mode))
    }

    #[rstest]
    #[case("unknown")]
    fn unknown_str_cannot_be_converted_to_drawing_mode(#[case] unknown_str: &str) {
        assert!(matches!(
            DrawingMode::from_str(unknown_str),
            Err(strum::ParseError::VariantNotFound)
        ))
    }

    #[rstest]
    #[case("yokohama-to-tokyo____".into(), "follow_road".into(), "{inwE}uesYcoh@u|Z".into(), Segment::yokohama_to_tokyo(false, None, false, DrawingMode::FollowRoad))]
    fn pair_of_string_can_be_converted_to_segment(
        #[case] id: String,
        #[case] mode: String,
        #[case] polyline: String,
        #[case] expected_seg: Segment,
    ) {
        let convert_result = Segment::try_from((id.clone(), mode, polyline));
        assert_eq!(convert_result, Ok(expected_seg));
        assert_eq!(convert_result.unwrap().id.to_string(), id);
    }

    macro_rules! point_segment {
        ($fix_name:ident, $set_ele:expr, $dist_offset:expr, $init_empty:expr, $mode:expr) => {
            Segment {
                id: SegmentId::from_string(format!("{:_<21}", stringify!($fix_name))),
                start: Coordinate::$fix_name(false, None),
                goal: Coordinate::$fix_name(false, None),
                points: if $init_empty {
                    Vec::new()
                } else {
                    vec![Coordinate::$fix_name($set_ele, $dist_offset)]
                },
                mode: $mode,
            }
        };
    }

    pub trait SegmentFixtures {
        fn yokohama_to_tokyo(
            set_ele: bool,
            dist_offset: Option<f64>,
            init_empty: bool,
            mode: DrawingMode,
        ) -> Segment {
            Segment {
                id: SegmentId::from_string("yokohama-to-tokyo____".into()),
                start: Coordinate::yokohama(false, None),
                goal: Coordinate::tokyo(false, None),
                points: if init_empty {
                    Vec::new()
                } else {
                    Coordinate::yokohama_to_tokyo_coords(set_ele, dist_offset)
                },
                mode,
            }
        }

        fn tokyo_to_chiba(
            set_ele: bool,
            dist_offset: Option<f64>,
            init_empty: bool,
            mode: DrawingMode,
        ) -> Segment {
            Segment {
                id: SegmentId::from_string("tokyo-to-chiba_______".into()),
                start: Coordinate::tokyo(false, None),
                goal: Coordinate::chiba(false, None),
                points: if init_empty {
                    Vec::new()
                } else {
                    Coordinate::tokyo_to_chiba_coords(set_ele, dist_offset)
                },
                mode,
            }
        }

        fn yokohama_to_chiba(
            set_ele: bool,
            dist_offset: Option<f64>,
            init_empty: bool,
            mode: DrawingMode,
        ) -> Segment {
            Segment {
                id: SegmentId::from_string("yokohama-to-chiba____".into()),
                start: Coordinate::yokohama(false, None),
                goal: Coordinate::chiba(false, None),
                points: if init_empty {
                    Vec::new()
                } else {
                    Coordinate::yokohama_to_chiba_coords(set_ele, dist_offset)
                },
                mode,
            }
        }

        fn yokohama(
            set_ele: bool,
            dist_offset: Option<f64>,
            init_empty: bool,
            mode: DrawingMode,
        ) -> Segment {
            point_segment!(yokohama, set_ele, dist_offset, init_empty, mode)
        }

        fn tokyo(
            set_ele: bool,
            dist_offset: Option<f64>,
            init_empty: bool,
            mode: DrawingMode,
        ) -> Segment {
            point_segment!(tokyo, set_ele, dist_offset, init_empty, mode)
        }

        fn chiba(
            set_ele: bool,
            dist_offset: Option<f64>,
            init_empty: bool,
            mode: DrawingMode,
        ) -> Segment {
            point_segment!(chiba, set_ele, dist_offset, init_empty, mode)
        }
    }

    impl SegmentFixtures for Segment {}
}
