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

#[derive(Clone, Debug, Serialize, Getters, PartialEq)]
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

#[cfg(test)]
pub(crate) mod tests {
    use rstest::rstest;

    use crate::model::route::coordinate::tests::CoordinateFixtures;
    use crate::model::route::segment_list::segment::tests::SegmentFixtures;

    use super::*;

    pub trait SegmentListFixture {
        fn step0_empty() -> SegmentList {
            SegmentList {
                segments: vec![],
                removed_range: None,
                inserted_range: None,
            }
        }

        fn step1_add_yokohama(set_ele: bool, set_dist: bool, after_op: bool) -> SegmentList {
            SegmentList {
                segments: if after_op {
                    vec![Segment::new_empty(
                        Coordinate::yokohama(),
                        Coordinate::yokohama(),
                    )]
                } else {
                    vec![Segment::yokohama_end(set_ele, set_dist)]
                },
                removed_range: after_op.then(|| 0..0),
                inserted_range: after_op.then(|| 0..1),
            }
        }

        fn step2_add_chiba(set_ele: bool, set_dist: bool, after_op: bool) -> SegmentList {
            SegmentList {
                segments: if after_op {
                    vec![
                        Segment::new_empty(Coordinate::yokohama(), Coordinate::chiba()),
                        Segment::new_empty(Coordinate::chiba(), Coordinate::chiba()),
                    ]
                } else {
                    vec![
                        Segment::yokohama_to_chiba(set_ele, set_dist),
                        Segment::yokohama_to_chiba_end(set_ele, set_dist),
                    ]
                },
                removed_range: after_op.then(|| 0..1),
                inserted_range: after_op.then(|| 0..2),
            }
        }

        fn step3_add_tokyo(set_ele: bool, set_dist: bool, after_op: bool) -> SegmentList {
            SegmentList {
                segments: if after_op {
                    vec![
                        Segment::new_empty(Coordinate::yokohama(), Coordinate::tokyo()),
                        Segment::new_empty(Coordinate::tokyo(), Coordinate::chiba()),
                        Segment::yokohama_to_chiba_via_tokyo_end(false, false),
                    ]
                } else {
                    vec![
                        Segment::yokohama_to_tokyo(set_ele, set_dist),
                        Segment::tokyo_to_chiba(set_ele, set_dist),
                        Segment::yokohama_to_chiba_via_tokyo_end(set_ele, set_dist),
                    ]
                },
                removed_range: after_op.then(|| 0..1),
                inserted_range: after_op.then(|| 0..2),
            }
        }

        fn step4_rm_tokyo(set_ele: bool, set_dist: bool, after_op: bool) -> SegmentList {
            SegmentList {
                segments: if after_op {
                    vec![
                        Segment::new_empty(Coordinate::yokohama(), Coordinate::chiba()),
                        Segment::yokohama_to_chiba_end(false, false),
                    ]
                } else {
                    vec![
                        Segment::yokohama_to_chiba(set_ele, set_dist),
                        Segment::yokohama_to_chiba_end(set_ele, set_dist),
                    ]
                },
                removed_range: after_op.then(|| 0..2),
                inserted_range: after_op.then(|| 0..1),
            }
        }

        fn step5_rm_chiba(set_ele: bool, set_dist: bool, after_op: bool) -> SegmentList {
            SegmentList {
                segments: if after_op {
                    vec![Segment::new_empty(
                        Coordinate::yokohama(),
                        Coordinate::yokohama(),
                    )]
                } else {
                    vec![Segment::yokohama_end(set_ele, set_dist)]
                },
                removed_range: after_op.then(|| 0..2),
                inserted_range: after_op.then(|| 0..1),
            }
        }

        fn step6_rm_yokohama(after_op: bool) -> SegmentList {
            SegmentList {
                segments: vec![],
                removed_range: after_op.then(|| 0..1),
                inserted_range: after_op.then(|| 0..0),
            }
        }

        fn step7_init_yokohama_to_chiba(
            set_ele: bool,
            set_dist: bool,
            after_op: bool,
        ) -> SegmentList {
            SegmentList {
                segments: if after_op {
                    vec![
                        Segment::new_empty(Coordinate::yokohama(), Coordinate::chiba()),
                        Segment::new_empty(Coordinate::chiba(), Coordinate::chiba()),
                    ]
                } else {
                    vec![
                        Segment::yokohama_to_chiba(set_ele, set_dist),
                        Segment::yokohama_to_chiba_end(set_ele, set_dist),
                    ]
                },
                removed_range: after_op.then(|| 0..0),
                inserted_range: after_op.then(|| 0..3),
            }
        }

        fn step8_mv_chiba_to_tokyo(set_ele: bool, set_dist: bool, after_op: bool) -> SegmentList {
            SegmentList {
                segments: if after_op {
                    vec![
                        Segment::new_empty(Coordinate::yokohama(), Coordinate::tokyo()),
                        Segment::new_empty(Coordinate::tokyo(), Coordinate::tokyo()),
                    ]
                } else {
                    vec![
                        Segment::yokohama_to_tokyo(set_ele, set_dist),
                        Segment::yokohama_to_tokyo_end(set_ele, set_dist),
                    ]
                },
                removed_range: after_op.then(|| 1..3),
                inserted_range: after_op.then(|| 1..3),
            }
        }

        fn step9_mv_tokyo_to_chiba(set_ele: bool, set_dist: bool, after_op: bool) -> SegmentList {
            SegmentList {
                segments: if after_op {
                    vec![
                        Segment::new_empty(Coordinate::yokohama(), Coordinate::chiba()),
                        Segment::new_empty(Coordinate::chiba(), Coordinate::chiba()),
                    ]
                } else {
                    vec![
                        Segment::yokohama_to_chiba(set_ele, set_dist),
                        Segment::yokohama_to_chiba_end(set_ele, set_dist),
                    ]
                },
                removed_range: after_op.then(|| 1..3),
                inserted_range: after_op.then(|| 1..3),
            }
        }

        fn step10_clear_all(after_op: bool) -> SegmentList {
            SegmentList {
                segments: vec![],
                removed_range: after_op.then(|| 0..3),
                inserted_range: after_op.then(|| 0..0),
            }
        }
    }

    impl SegmentListFixture for SegmentList {}

    #[rstest]
    #[case::empty(SegmentList::step0_empty(), 0.)]
    #[case::single_point(SegmentList::step1_add_yokohama(false, true, false), 0.)]
    #[case::yokohama_to_chiba(SegmentList::step2_add_chiba(false, true, false), 61926.0425172123)]
    fn can_get_total_distance(#[case] seg_list: SegmentList, #[case] expected_distance: f64) {
        assert_eq!(
            seg_list.get_total_distance().unwrap().value(),
            expected_distance
        )
    }

    #[rstest]
    #[case::empty(SegmentList::step0_empty(), 0)]
    #[case::single_point(SegmentList::step1_add_yokohama(true, false, false), 0)]
    #[case::yokohama_to_chiba(SegmentList::step2_add_chiba(true, false, false), 10)]
    fn can_calc_elevation_gain(#[case] seg_list: SegmentList, #[case] expected_gain: i32) {
        assert_eq!(seg_list.calc_elevation_gain().value(), expected_gain)
    }

    #[rstest]
    #[case::empty(SegmentList::step0_empty(), SegmentList::step0_empty())]
    #[case::single_point(
        SegmentList::step1_add_yokohama(false, false, false),
        SegmentList::step1_add_yokohama(false, true, false)
    )]
    #[case::yokohama_to_chiba(
        SegmentList::step2_add_chiba(false, false, false),
        SegmentList::step2_add_chiba(false, true, false)
    )]
    fn can_attach_distances(
        #[case] mut seg_list_without_dist: SegmentList,
        #[case] expected_seg_list: SegmentList,
    ) {
        seg_list_without_dist.attach_distance_from_start().unwrap();
        assert_eq!(seg_list_without_dist, expected_seg_list)
    }

    #[rstest]
    #[case::single_point(
        SegmentList::step1_add_yokohama(false, false, true),
        &[Segment::new_empty(Coordinate::yokohama(), Coordinate::yokohama())]
    )]
    #[case::yokohama_to_chiba_via_tokyo(
        SegmentList::step3_add_tokyo(false, false, true),
        &[
            Segment::new_empty(Coordinate::yokohama(), Coordinate::tokyo()),
            Segment::new_empty(Coordinate::tokyo(), Coordinate::chiba())
        ]
    )]
    fn can_get_inserted_slice(#[case] seg_list: SegmentList, #[case] expected_slice: &[Segment]) {
        assert_eq!(seg_list.get_inserted_slice().unwrap(), expected_slice)
    }

    #[rstest]
    #[case::yokohama_to_chiba(SegmentList::step2_add_chiba(false, false, false))]
    fn cannot_get_inserted_slice_from_unchanged(#[case] seg_list: SegmentList) {
        assert!(matches!(
            seg_list.get_inserted_slice(),
            Err(ApplicationError::DomainError(_))
        ))
    }

    #[rstest]
    #[case::empty(SegmentList::step0_empty(), vec![])]
    #[case::single_point(
        SegmentList::step1_add_yokohama(false, false, false),
        vec![Coordinate::yokohama()]
    )]
    #[case::yokohama_to_chiba_via_tokyo(
        SegmentList::step3_add_tokyo(false, false, false),
        vec![Coordinate::yokohama(), Coordinate::tokyo(), Coordinate::chiba()]
    )]
    fn can_gather_waypoints(
        #[case] seg_list: SegmentList,
        #[case] expected_waypoints: Vec<Coordinate>,
    ) {
        assert_eq!(seg_list.gather_waypoints(), expected_waypoints)
    }

    #[rstest]
    #[case::empty(SegmentList::step0_empty(), vec![])]
    #[case::single_point(
        SegmentList::step1_add_yokohama(false, false, false),
        vec![]
    )]
    #[case::yokohama_to_chiba_via_tokyo(
        SegmentList::step3_add_tokyo(false, false, false),
        vec![Segment::yokohama_to_tokyo(false, false), Segment::tokyo_to_chiba(false, false)]
    )]
    fn can_convert_into_segments_in_between(
        #[case] seg_list: SegmentList,
        #[case] expected_segments: Vec<Segment>,
    ) {
        assert_eq!(seg_list.into_segments_in_between(), expected_segments)
    }

    // NOTE: replace_rangeは，これを呼び出すOperation::applyのテストで検証する
}
