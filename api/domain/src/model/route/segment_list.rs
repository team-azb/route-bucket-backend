use std::cmp::max;
use std::convert::TryInto;
use std::ops::RangeBounds;
use std::slice::{Iter, IterMut};

use geo::algorithm::haversine_distance::HaversineDistance;
use getset::Getters;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::Serialize;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use super::bounding_box::BoundingBox;
use super::coordinate::Coordinate;
use super::types::{Distance, Elevation};

pub use self::segment::{DrawingMode, Segment};

mod segment;

#[derive(Clone, Debug, Serialize, Getters)]
#[get = "pub"]
#[cfg_attr(any(test, feature = "fixtures"), derive(PartialEq))]
pub struct SegmentList {
    pub(super) segments: Vec<Segment>,
}

impl SegmentList {
    pub fn get_total_distance(&self) -> ApplicationResult<Distance> {
        if let Some(last_seg) = self.segments.last() {
            let last_point = last_seg.points.last().ok_or_else(|| {
                ApplicationError::DomainError(format!(
                    "Last segment cannot be empty at get_total_distance! ({:?})",
                    last_seg
                ))
            })?;
            (*last_point.distance_from_start()).ok_or_else(|| {
                ApplicationError::DomainError(format!(
                    "Failed to calculate total distance. {:?}",
                    self
                ))
            })
        } else {
            Ok(Distance::zero())
        }
    }

    pub fn calc_elevation_gain(&self) -> Elevation {
        self.iter()
            .par_bridge()
            .fold(Elevation::zero, |total_gain, seg| {
                let mut gain = Elevation::zero();
                let mut prev_elev = Elevation::max_value();
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
                    *prev_op = Some(coord.clone());
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

    pub fn calc_bounding_box(&self) -> ApplicationResult<BoundingBox> {
        let mut coord_iter = self.iter().map(|seg| seg.iter()).flatten();

        if let Some(first_coord) = coord_iter.next() {
            let mut min_coord = first_coord.clone();
            let mut max_coord = first_coord.clone();
            min_coord.elevation = None;
            min_coord.distance_from_start = None;
            max_coord.elevation = None;
            max_coord.distance_from_start = None;

            coord_iter.for_each(|coord| {
                min_coord.latitude = std::cmp::min(min_coord.latitude, coord.latitude);
                min_coord.longitude = std::cmp::min(min_coord.longitude, coord.longitude);
                max_coord.latitude = std::cmp::max(max_coord.latitude, coord.latitude);
                max_coord.longitude = std::cmp::max(max_coord.longitude, coord.longitude);
            });
            Ok((min_coord, max_coord).into())
        } else {
            Err(ApplicationError::DomainError(
                "Cannot calc_bounding_box on an empty SegmentList".into(),
            ))
        }
    }

    pub fn gather_waypoints(&self) -> Vec<Coordinate> {
        self.segments.iter().map(|seg| seg.start.clone()).collect()
    }

    pub fn into_segments_in_between(self) -> Vec<Segment> {
        let mut segments: Vec<Segment> = self.into();
        if !segments.is_empty() {
            segments.pop();
        }
        segments
    }

    // methods from Vec<Segment>

    pub fn iter(&self) -> Iter<Segment> {
        self.segments.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Segment> {
        self.segments.iter_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }

    pub fn splice<R, I>(&mut self, range: R, replace_with: I)
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = Segment>,
    {
        self.segments.splice(range, replace_with);
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

#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod tests {
    use rstest::{fixture, rstest};

    #[cfg(test)]
    use crate::model::route::bounding_box::tests::BoundingBoxFixture;
    #[cfg(test)]
    use crate::model::route::coordinate::tests::CoordinateFixtures;
    pub use crate::model::route::segment_list::segment::tests::SegmentFixtures;

    use super::*;

    #[fixture]
    fn yokohama_empty() -> SegmentList {
        SegmentList::yokohama(false, false, true)
    }

    #[fixture]
    fn yokohama_verbose() -> SegmentList {
        SegmentList::yokohama(true, true, false)
    }

    #[fixture]
    fn yokohama_to_chiba_empty() -> SegmentList {
        SegmentList::yokohama_to_chiba(false, false, true)
    }

    #[fixture]
    fn yokohama_to_chiba_via_tokyo_empty() -> SegmentList {
        SegmentList::yokohama_to_chiba_via_tokyo(false, false, true)
    }

    #[fixture]
    fn yokohama_to_chiba_via_tokyo_verbose() -> SegmentList {
        SegmentList::yokohama_to_chiba_via_tokyo(true, true, false)
    }

    #[rstest]
    #[case::empty(SegmentList::empty(), 0.)]
    #[case::single_point(yokohama_verbose(), 0.)]
    #[case::yokohama_to_chiba(yokohama_to_chiba_via_tokyo_verbose(), 58759.973932514884)]
    fn can_get_total_distance(#[case] seg_list: SegmentList, #[case] expected_distance: f64) {
        assert_eq!(
            seg_list.get_total_distance().unwrap().value(),
            expected_distance
        )
    }

    #[rstest]
    fn cannot_get_total_distance_if_empty(
        #[from(yokohama_to_chiba_via_tokyo_empty)] empty_seg_list: SegmentList,
    ) {
        assert!(matches!(
            empty_seg_list.get_total_distance(),
            Err(ApplicationError::DomainError(_))
        ))
    }

    #[rstest]
    #[case::empty(SegmentList::empty(), 0)]
    #[case::single_point(yokohama_verbose(), 0)]
    #[case::yokohama_to_chiba(yokohama_to_chiba_via_tokyo_verbose(), 10)]
    #[case::yokohama_to_chiba_empty(yokohama_to_chiba_via_tokyo_empty(), 0)]
    fn can_calc_elevation_gain(#[case] seg_list: SegmentList, #[case] expected_gain: i32) {
        assert_eq!(seg_list.calc_elevation_gain().value(), expected_gain)
    }

    #[rstest]
    #[case::empty(SegmentList::empty(), SegmentList::empty())]
    #[case::single_point(
        SegmentList::yokohama(false, false, false),
        SegmentList::yokohama(false, true, false)
    )]
    #[case::yokohama_to_chiba(
        SegmentList::yokohama_to_chiba(false, false, false),
        SegmentList::yokohama_to_chiba(false, true, false)
    )]
    fn can_attach_distances(
        #[case] mut seg_list_without_dist: SegmentList,
        #[case] expected_seg_list: SegmentList,
    ) {
        seg_list_without_dist.attach_distance_from_start().unwrap();
        assert_eq!(seg_list_without_dist, expected_seg_list)
    }

    #[rstest]
    #[case::single_point(SegmentList::yokohama(false, false, false), BoundingBox::yokohama())]
    #[case::yokohama_to_chiba_via_tokyo(
        SegmentList::yokohama_to_chiba_via_tokyo(false, false, false),
        BoundingBox::yokohama_to_chiba_via_tokyo()
    )]
    fn can_calc_bounding_box(#[case] seg_list: SegmentList, #[case] expected_bbox: BoundingBox) {
        assert_eq!(seg_list.calc_bounding_box(), Ok(expected_bbox))
    }

    #[rstest]
    fn cannot_calc_bounding_box_if_empty(
        #[from(yokohama_to_chiba_via_tokyo_empty)] empty_seg_list: SegmentList,
    ) {
        assert!(matches!(
            empty_seg_list.calc_bounding_box(),
            Err(ApplicationError::DomainError(_))
        ))
    }

    #[rstest]
    #[case::empty(SegmentList::empty(), vec![])]
    #[case::single_point(
        yokohama_empty(),
        vec![Coordinate::yokohama(false, None)]
    )]
    #[case::yokohama_to_chiba_via_tokyo(
        yokohama_to_chiba_via_tokyo_empty(),
        Coordinate::yokohama_to_chiba_via_tokyo_coords(false, None)
    )]
    fn can_gather_waypoints(
        #[case] seg_list: SegmentList,
        #[case] expected_waypoints: Vec<Coordinate>,
    ) {
        assert_eq!(seg_list.gather_waypoints(), expected_waypoints)
    }

    #[rstest]
    #[case::empty(SegmentList::empty(), vec![])]
    #[case::single_point(
        SegmentList::yokohama(false, false, false),
        vec![]
    )]
    #[case::yokohama_to_chiba_via_tokyo(
        yokohama_to_chiba_via_tokyo_verbose(),
        vec![
            Segment::yokohama_to_tokyo(true, Some(0.), false, DrawingMode::Freehand),
            Segment::tokyo_to_chiba(true, Some(26936.42633640023), false, DrawingMode::Freehand)
        ]
    )]
    fn can_convert_into_segments_in_between(
        #[case] seg_list: SegmentList,
        #[case] expected_segments: Vec<Segment>,
    ) {
        assert_eq!(seg_list.into_segments_in_between(), expected_segments)
    }

    pub trait SegmentListFixture {
        fn empty() -> SegmentList {
            SegmentList { segments: vec![] }
        }

        fn yokohama(set_ele: bool, set_dist: bool, empty: bool) -> SegmentList {
            SegmentList {
                segments: vec![Segment::yokohama(
                    set_ele,
                    set_dist.then(|| 0.),
                    empty,
                    DrawingMode::FollowRoad,
                )],
            }
        }

        fn yokohama_to_tokyo(set_ele: bool, set_dist: bool, empty: bool) -> SegmentList {
            SegmentList {
                segments: vec![
                    Segment::yokohama_to_tokyo(
                        set_ele,
                        set_dist.then(|| 0.),
                        empty,
                        DrawingMode::Freehand,
                    ),
                    Segment::tokyo(
                        set_ele,
                        set_dist.then(|| 26936.42633640023),
                        empty,
                        DrawingMode::Freehand,
                    ),
                ],
            }
        }

        fn yokohama_to_chiba(set_ele: bool, set_dist: bool, empty: bool) -> SegmentList {
            SegmentList {
                segments: vec![
                    Segment::yokohama_to_chiba(
                        set_ele,
                        set_dist.then(|| 0.),
                        empty,
                        DrawingMode::FollowRoad,
                    ),
                    Segment::chiba(
                        set_ele,
                        set_dist.then(|| 46779.709825324135),
                        empty,
                        DrawingMode::FollowRoad,
                    ),
                ],
            }
        }

        fn yokohama_to_chiba_via_tokyo(set_ele: bool, set_dist: bool, empty: bool) -> SegmentList {
            SegmentList {
                segments: vec![
                    Segment::yokohama_to_tokyo(
                        set_ele,
                        set_dist.then(|| 0.),
                        empty,
                        DrawingMode::Freehand,
                    ),
                    Segment::tokyo_to_chiba(
                        set_ele,
                        set_dist.then(|| 26936.42633640023),
                        empty,
                        DrawingMode::Freehand,
                    ),
                    Segment::chiba(
                        set_ele,
                        set_dist.then(|| 58759.973932514884),
                        empty,
                        DrawingMode::FollowRoad,
                    ),
                ],
            }
        }
    }

    impl SegmentListFixture for SegmentList {}
}
