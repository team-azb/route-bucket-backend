use std::convert::TryFrom;
use std::slice::{Iter, IterMut};
use std::str::FromStr;

use derive_more::IntoIterator;
use geo::prelude::HaversineDistance;
use getset::Getters;
use num_traits::Zero;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};

use route_bucket_utils::{ApplicationError, ApplicationResult};

use super::super::coordinate::Coordinate;
use super::super::types::{Distance, Polyline};
use crate::model::types::NanoId;

pub(crate) type SegmentId = NanoId<Segment, 21>;

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

    pub(super) fn calc_distance_from_start(&mut self) {
        self.iter_mut()
            .scan((Distance::zero(), None), |(dist, prev_op), coord| {
                if let Some(prev_coord) = prev_op {
                    *dist += coord.haversine_distance(prev_coord);
                }
                coord.set_distance_from_start(*dist);
                *prev_op = Some(coord.clone());
                Some(*dist)
            })
            .last()
            .unwrap();
    }

    pub(super) fn get_distance_offset(&self) -> Distance {
        self.points
            .first()
            .and_then(|coord| *coord.distance_from_start())
            .unwrap_or_else(Distance::zero)
    }

    pub(super) fn set_distance_offset(&mut self, offset: Distance) {
        let org_offset = self.get_distance_offset();
        if org_offset != offset {
            self.points.iter_mut().par_bridge().for_each(|coord| {
                if let Some(org_distance) = coord.distance_from_start {
                    coord.set_distance_from_start(org_distance - org_offset + offset);
                }
            });
        }
    }

    pub(super) fn has_distance(&self) -> bool {
        self.points
            .first()
            .and_then(|coord| *coord.distance_from_start())
            .is_some()
    }

    pub fn get_distance(&self) -> Distance {
        self.points
            .last()
            .and_then(|coord| *coord.distance_from_start())
            .unwrap_or_else(Distance::zero)
    }

    pub fn set_points(&mut self, mut points: Vec<Coordinate>) -> ApplicationResult<()> {
        if self.is_empty() {
            match points.first() {
                Some(coord) if *coord == self.start => (),
                _ => {
                    points.insert(0, self.start.clone());
                }
            }
            match points.last() {
                Some(coord) if *coord == self.goal => (),
                _ => {
                    points.push(self.goal.clone());
                }
            }
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
        Segment::yokohama_to_tokyo(false, None, false, DrawingMode::FollowRoad)
    }

    #[fixture]
    fn yokohama_to_tokyo_with_distance() -> Segment {
        yokohama_to_tokyo_with_distance_offset(0.)
    }

    fn yokohama_to_tokyo_with_distance_offset(offset: f64) -> Segment {
        Segment::yokohama_to_tokyo(false, Some(offset), false, DrawingMode::FollowRoad)
    }

    #[fixture]
    fn yokohama_to_tokyo_verbose() -> Segment {
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
    #[case(yokohama_to_tokyo(), yokohama_to_tokyo_with_distance())]
    fn can_calc_distance_from_start(#[case] mut org_seg: Segment, #[case] expected_seg: Segment) {
        org_seg.calc_distance_from_start();
        assert_eq!(org_seg, expected_seg)
    }

    #[rstest]
    #[case::no_offset(yokohama_to_tokyo_with_distance(), 0.)]
    #[case::has_offset(yokohama_to_tokyo_with_distance_offset(1234.56), 1234.56)]
    fn can_get_distance_offset(#[case] seg: Segment, #[case] expected: f64) {
        assert_eq!(seg.get_distance_offset().value(), expected)
    }

    #[rstest]
    #[case::init_offset(
        yokohama_to_tokyo_with_distance(),
        1234.56,
        yokohama_to_tokyo_with_distance_offset(1234.56)
    )]
    #[case::reset_offset(
        yokohama_to_tokyo_with_distance_offset(32.1),
        12.3,
        yokohama_to_tokyo_with_distance_offset(12.3)
    )]
    fn set_distance_offset(
        #[case] mut org_seg: Segment,
        #[case] offset: f64,
        #[case] expected_seg: Segment,
    ) {
        org_seg.set_distance_offset(Distance::try_from(offset).unwrap());
        assert_eq!(org_seg, expected_seg)
    }

    #[rstest]
    #[case::empty(yokohama_to_tokyo_empty(), false)]
    #[case::no_distance(yokohama_to_tokyo_empty(), false)]
    #[case::has_distance(yokohama_to_tokyo_with_distance(), true)]
    fn can_check_has_distance(#[case] org_seg: Segment, #[case] expected: bool) {
        assert_eq!(org_seg.has_distance(), expected)
    }

    #[rstest]
    #[case::empty_segment(yokohama_to_tokyo_empty(), 0.)]
    #[case::yokohama_to_tokyo(yokohama_to_tokyo_verbose(), 26936.42633640023)]
    fn can_return_segment_distance(#[case] seg: Segment, #[case] expected: f64) {
        assert_eq!(seg.get_distance().value(), expected)
    }

    #[rstest]
    #[case::set_empty(Vec::new())]
    #[case::set_yokohama_to_tokyo(Coordinate::yokohama_to_tokyo_coords(false, None))]
    fn can_set_points(
        #[from(yokohama_to_tokyo_empty)] mut empty_seg: Segment,
        #[case] points: Vec<Coordinate>,
        #[from(yokohama_to_tokyo)] expected_segment: Segment,
    ) {
        empty_seg.set_points(points).unwrap();
        assert_eq!(&empty_seg, &expected_segment)
    }

    #[rstest]
    fn cannot_set_points_twice(
        #[from(yokohama_to_tokyo_verbose)] mut filled_seg: Segment,
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
    #[case("yokohama-to-tokyo____".into(), "follow_road".into(), "{inwE}uesYcoh@u|Z".into(), yokohama_to_tokyo())]
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
