use std::cmp::max;
use std::convert::TryInto;
use std::slice::{Iter, IterMut};

use geo::algorithm::haversine_distance::HaversineDistance;
use getset::Getters;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::Serialize;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::{Distance, Elevation};

use super::coordinate::Coordinate;

pub use self::segment::Segment;

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

    pub fn insert_waypoint(&mut self, pos: usize, coord: Coordinate) -> ApplicationResult<()> {
        let org_len = self.segments.len();
        if pos <= org_len {
            if pos == 0 {
                let goal = self
                    .segments
                    .first()
                    .map(|seg| seg.start.clone())
                    .unwrap_or(coord.clone());
                self.segments.insert(0, Segment::new_empty(coord, goal));
            } else {
                let org_seg = self.segments.remove(pos - 1);
                let start = org_seg.start.clone();
                let goal = if pos == org_len {
                    coord.clone()
                } else {
                    org_seg.goal
                };
                self.segments
                    .insert(pos - 1, Segment::new_empty(start, coord.clone()));
                self.segments.insert(pos, Segment::new_empty(coord, goal));
            }
            Ok(())
        } else {
            Err(ApplicationError::DomainError(format!(
                "pos({}) cannot be greater than segments.len()({}) at SegmentList::insert_waypoint",
                pos,
                self.segments.len()
            )))
        }
    }

    pub fn remove_waypoint(&mut self, pos: usize) -> ApplicationResult<()> {
        let org_len = self.segments.len();
        if org_len == 0 {
            return Err(ApplicationError::DomainError(
                "segments.len() cannot be 0 at SegmentList::remove_waypoint".into(),
            ));
        }

        if pos < org_len {
            let org_second_seg = self.segments.remove(pos);
            if pos > 0 {
                let org_first_seg = self.segments.remove(pos - 1);
                let start = org_first_seg.start.clone();
                let goal = if pos == org_len - 1 {
                    org_first_seg.start
                } else {
                    org_second_seg.goal
                };
                self.segments
                    .insert(pos - 1, Segment::new_empty(start, goal));
            }
            Ok(())
        } else {
            Err(ApplicationError::DomainError(format!(
                "pos({}) cannot be greater than or equal to segments.len()({}) at SegmentList::remove_waypoint",
                pos,
                self.segments.len()
            )))
        }
    }

    pub fn move_waypoint(&mut self, pos: usize, coord: Coordinate) -> ApplicationResult<()> {
        self.remove_waypoint(pos)?;
        self.insert_waypoint(pos, coord)?;

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
    #[case::front(
        SegmentList::tokyo_to_chiba(false, false, true),
        0,
        Coordinate::yokohama(false, None)
    )]
    #[case::middle(
        SegmentList::yokohama_to_chiba(false, false, true),
        1,
        Coordinate::tokyo(false, None)
    )]
    #[case::back(
        SegmentList::yokohama_to_tokyo(false, false, true),
        2,
        Coordinate::chiba(false, None)
    )]
    fn can_insert_waypoint(
        #[case] mut seg_list: SegmentList,
        #[case] pos: usize,
        #[case] coord: Coordinate,
        #[from(yokohama_to_chiba_via_tokyo_empty)] expected_seg_list: SegmentList,
    ) {
        seg_list.insert_waypoint(pos, coord).unwrap();
        assert_eq!(seg_list, expected_seg_list)
    }

    #[rstest]
    #[case::front(0, SegmentList::tokyo_to_chiba(false, false, true))]
    #[case::middle(1, SegmentList::yokohama_to_chiba(false, false, true))]
    #[case::back(2, SegmentList::yokohama_to_tokyo(false, false, true))]
    fn can_remove_waypoint(
        #[from(yokohama_to_chiba_via_tokyo_empty)] mut seg_list: SegmentList,
        #[case] pos: usize,
        #[case] expected_seg_list: SegmentList,
    ) {
        seg_list.remove_waypoint(pos).unwrap();
        assert_eq!(seg_list, expected_seg_list)
    }

    #[rstest]
    #[case::front(
        SegmentList::tokyo_to_chiba(false, false, true),
        0,
        Coordinate::yokohama(false, None)
    )]
    #[case::back(
        SegmentList::yokohama_to_tokyo(false, false, true),
        1,
        Coordinate::chiba(false, None)
    )]
    fn can_move_waypoint(
        #[case] mut seg_list: SegmentList,
        #[case] pos: usize,
        #[case] coord: Coordinate,
        #[from(yokohama_to_chiba_empty)] expected_seg_list: SegmentList,
    ) {
        seg_list.move_waypoint(pos, coord).unwrap();
        assert_eq!(seg_list, expected_seg_list)
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
        vec![Segment::yokohama_to_tokyo(true, Some(0.), false), Segment::tokyo_to_chiba(true, Some(26936.42633640023), false)]
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
                segments: vec![Segment::yokohama(set_ele, set_dist.then(|| 0.), empty)],
            }
        }

        fn yokohama_to_tokyo(set_ele: bool, set_dist: bool, empty: bool) -> SegmentList {
            SegmentList {
                segments: vec![
                    Segment::yokohama_to_tokyo(set_ele, set_dist.then(|| 0.), empty),
                    Segment::tokyo(set_ele, set_dist.then(|| 26936.42633640023), empty),
                ],
            }
        }

        fn tokyo_to_chiba(set_ele: bool, set_dist: bool, empty: bool) -> SegmentList {
            SegmentList {
                segments: vec![
                    Segment::tokyo_to_chiba(set_ele, set_dist.then(|| 0.), empty),
                    Segment::chiba(set_ele, set_dist.then(|| 31823.54759611465), empty),
                ],
            }
        }

        fn yokohama_to_chiba(set_ele: bool, set_dist: bool, empty: bool) -> SegmentList {
            SegmentList {
                segments: vec![
                    Segment::yokohama_to_chiba(set_ele, set_dist.then(|| 0.), empty),
                    Segment::chiba(set_ele, set_dist.then(|| 46779.709825324135), empty),
                ],
            }
        }

        fn yokohama_to_chiba_via_tokyo(set_ele: bool, set_dist: bool, empty: bool) -> SegmentList {
            SegmentList {
                segments: vec![
                    Segment::yokohama_to_tokyo(set_ele, set_dist.then(|| 0.), empty),
                    Segment::tokyo_to_chiba(set_ele, set_dist.then(|| 26936.42633640023), empty),
                    Segment::chiba(set_ele, set_dist.then(|| 58759.973932514884), empty),
                ],
            }
        }
    }

    impl SegmentListFixture for SegmentList {}
}
