use std::cmp::max;
use std::slice::{Iter, IterMut};

use getset::Getters;
use num_traits::Zero;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::Serialize;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use super::bounding_box::BoundingBox;
use super::coordinate::Coordinate;
use super::types::{Distance, Elevation};

pub use self::operation::{Operation, OperationId, OperationType, SegmentTemplate};
pub use self::segment::{DrawingMode, Segment};

mod operation;
mod segment;

#[derive(Clone, Debug, Serialize, Getters)]
#[get = "pub"]
#[cfg_attr(any(test, feature = "fixtures"), derive(PartialEq))]
pub struct SegmentList {
    pub(super) segments: Vec<Segment>,
}

impl SegmentList {
    pub fn apply_operation(&mut self, op: Operation) -> ApplicationResult<()> {
        self.segments.splice(
            op.pos..op.pos + op.org_seg_templates.len(),
            op.new_seg_templates
                .iter()
                .map(Clone::clone)
                .map(Segment::from),
        );
        Ok(())
    }

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

    pub fn calc_elevation_gain(&self) -> (Elevation, Elevation) {
        let gain_tuple_identity = || (Elevation::zero(), Elevation::zero());
        let gain_tuple_add = |(asc0, desc0), (asc1, desc1)| (asc0 + asc1, desc0 + desc1);
        self.iter()
            .par_bridge()
            .fold(gain_tuple_identity, |(ascent_total, descent_total), seg| {
                let mut ascent_gain = Elevation::zero();
                let mut descent_gain = Elevation::zero();
                let mut prev_elev = None;
                seg.iter().for_each(|coord| {
                    if let Some(elev) = coord.elevation() {
                        if let Some(prev_elev_value) = prev_elev {
                            ascent_gain += max(*elev - prev_elev_value, Elevation::zero());
                            descent_gain += max(prev_elev_value - *elev, Elevation::zero());
                        }
                        prev_elev = Some(*elev);
                    }
                });
                gain_tuple_add((ascent_total, descent_total), (ascent_gain, descent_gain))
            })
            .reduce(gain_tuple_identity, gain_tuple_add)
    }

    pub fn attach_distance_from_start(&mut self) {
        // compute cumulative distance within the segments
        self.iter_mut()
            .par_bridge()
            .filter(|seg| !seg.has_distance())
            .for_each(Segment::calc_distance_from_start);

        // convert to global cumulative distance
        self.iter_mut()
            .scan(Distance::zero(), |offset, seg| {
                let prev_offset = *offset;
                *offset += seg.get_distance();
                Some((seg, prev_offset))
            })
            .par_bridge()
            .for_each(|(seg, offset)| {
                seg.set_distance_offset(offset);
            });
    }

    pub fn calc_bounding_box(&self) -> ApplicationResult<BoundingBox> {
        let mut coord_iter = self.iter().flat_map(|seg| seg.iter());

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
    pub use crate::model::route::segment_list::operation::tests::OperationFixtures;
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
    #[case::add(
        Operation::add_tokyo(),
        yokohama_to_chiba_empty(),
        yokohama_to_chiba_via_tokyo_empty()
    )]
    #[case::remove(
        Operation::remove_tokyo(),
        yokohama_to_chiba_via_tokyo_empty(),
        yokohama_to_chiba_empty()
    )]
    #[case::move_(
        Operation::move_chiba_to_tokyo(),
        yokohama_to_chiba_empty(),
        SegmentList::yokohama_to_tokyo(false, false, true)
    )]
    fn can_apply_operation(
        #[case] op: Operation,
        #[case] mut seg_list: SegmentList,
        #[case] expected: SegmentList,
    ) {
        seg_list.apply_operation(op).unwrap();
        assert_eq!(seg_list, expected)
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
    #[case::empty(SegmentList::empty(), 0, 0)]
    #[case::single_point(yokohama_verbose(), 0, 0)]
    #[case::yokohama_to_chiba(yokohama_to_chiba_via_tokyo_verbose(), 10, 0)]
    #[case::yokohama_to_chiba_empty(yokohama_to_chiba_via_tokyo_empty(), 0, 0)]
    fn can_calc_elevation_gain(
        #[case] seg_list: SegmentList,
        #[case] expected_asc_gain: i32,
        #[case] expected_desc_gain: i32,
    ) {
        let (asc_gain, desc_gain) = seg_list.calc_elevation_gain();
        assert_eq!(asc_gain.value(), expected_asc_gain);
        assert_eq!(desc_gain.value(), expected_desc_gain)
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
        seg_list_without_dist.attach_distance_from_start();
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
