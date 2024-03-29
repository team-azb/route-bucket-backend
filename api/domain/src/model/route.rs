use std::slice::IterMut;

use derive_more::{From, Into};
use getset::Getters;

use route_bucket_utils::{ApplicationError, ApplicationResult};

pub use self::bounding_box::BoundingBox;
pub use self::coordinate::Coordinate;
pub use self::route_gpx::RouteGpx;
pub use self::route_info::RouteInfo;
pub use self::search_query::RouteSearchQuery;
pub use self::segment_list::{
    DrawingMode, Operation, OperationId, OperationType, Segment, SegmentList, SegmentTemplate,
};
pub use self::types::{Distance, Elevation, Latitude, Longitude, Polyline};

use super::types::NanoId;

pub(crate) mod bounding_box;
pub(crate) mod coordinate;
pub(crate) mod route_gpx;
pub(crate) mod route_info;
pub(crate) mod search_query;
pub(crate) mod segment_list;
pub(crate) mod types;

pub type RouteId = NanoId<Route, 11>;

#[derive(Clone, Debug, From, Into, Getters)]
#[get = "pub"]
#[cfg_attr(any(test, feature = "fixtures"), derive(PartialEq))]
pub struct Route {
    info: RouteInfo,
    op_list: Vec<Operation>,
    seg_list: SegmentList,
}

impl Route {
    pub fn new(info: RouteInfo, op_list: Vec<Operation>, seg_list: SegmentList) -> Self {
        Self {
            info,
            op_list,
            seg_list,
        }
    }

    pub fn get_operation(&self, pos: usize) -> ApplicationResult<&Operation> {
        self.op_list.get(pos).ok_or_else(|| {
            ApplicationError::DomainError(format!(
                "Index {} out of range for RouteEditor.op_list!(len={})",
                pos,
                self.op_list.len()
            ))
        })
    }

    pub fn push_operation(&mut self, op: Operation) -> ApplicationResult<()> {
        // pos以降の要素は全て捨てる
        self.op_list.truncate(self.info.op_num);
        self.op_list.push(op);

        self.apply_operation(false)
    }

    pub fn redo_operation(&mut self) -> ApplicationResult<()> {
        if self.info.op_num < self.op_list.len() {
            self.apply_operation(false)
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to redo.",
            ))
        }
    }

    pub fn undo_operation(&mut self) -> ApplicationResult<()> {
        if self.info.op_num > 0 {
            self.apply_operation(true)
        } else {
            Err(ApplicationError::InvalidOperation(
                "No more operations to undo.",
            ))
        }
    }

    fn apply_operation(&mut self, reverse: bool) -> ApplicationResult<()> {
        let mut op;
        if reverse {
            self.info.op_num -= 1;
            op = self.get_operation(self.info.op_num)?.clone();
            op.reverse()
        } else {
            op = self.get_operation(self.info.op_num)?.clone();
            self.info.op_num += 1;
        };

        self.seg_list.apply_operation(op)?;

        Ok(())
    }

    pub fn calc_route_features_from_seg_list(&mut self) -> ApplicationResult<()> {
        self.seg_list.attach_distance_from_start();

        let (asc_gain, desc_gain) = self.seg_list.calc_elevation_gain();
        self.info.ascent_elevation_gain = asc_gain;
        self.info.descent_elevation_gain = desc_gain;
        self.info.total_distance = self.seg_list.get_total_distance()?;

        Ok(())
    }

    pub fn iter_seg_mut(&mut self) -> IterMut<Segment> {
        self.seg_list.iter_mut()
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod tests {
    use rstest::{fixture, rstest};

    use crate::model::route::{
        route_info::tests::RouteInfoFixtures, segment_list::tests::OperationFixtures,
        segment_list::tests::SegmentListFixture,
    };

    use super::*;

    #[fixture]
    fn empty_route() -> Route {
        Route::empty()
    }

    #[fixture]
    fn full_route() -> Route {
        Route::yokohama_to_chiba_via_tokyo()
    }

    #[fixture]
    fn full_route_filled() -> Route {
        Route {
            info: RouteInfo::yokohama_to_chiba_via_tokyo(),
            ..Route::yokohama_to_chiba_via_tokyo_filled(true, true)
        }
    }

    #[fixture]
    fn full_route_filled_but_yet_to_calc_features() -> Route {
        Route::yokohama_to_chiba_via_tokyo_filled(true, false)
    }

    #[fixture]
    fn after_undo() -> Route {
        Route::yokohama_to_chiba_after_undo()
    }

    #[rstest]
    #[case::first(0, Operation::add_yokohama())]
    #[case::middle(1, Operation::add_chiba())]
    #[case::last(2, Operation::add_tokyo())]
    fn can_get_operation(full_route: Route, #[case] pos: usize, #[case] expected: Operation) {
        assert_eq!(full_route.get_operation(pos).unwrap().clone(), expected)
    }

    #[rstest]
    fn cannot_get_operation_out_of_range(full_route: Route) {
        assert!(matches!(
            full_route.get_operation(3),
            Err(ApplicationError::DomainError(_))
        ))
    }

    #[rstest]
    #[case::add(
        Route::yokohama_to_chiba(),
        Operation::add_tokyo(),
        Route::yokohama_to_chiba_via_tokyo()
    )]
    #[case::remove(
        Route::yokohama_to_chiba_via_tokyo(),
        Operation::remove_tokyo(),
        Route::yokohama_to_chiba_after_remove()
    )]
    #[case::move_(
        Route::yokohama_to_chiba(),
        Operation::move_chiba_to_tokyo(),
        Route::yokohama_to_tokyo()
    )]
    #[case::truncate_op_list(
        Route::yokohama_to_chiba_after_undo(),
        Operation::move_chiba_to_tokyo(),
        Route::yokohama_to_tokyo()
    )]
    fn can_push_operation(
        #[case] mut route: Route,
        #[case] op: Operation,
        #[case] expected: Route,
    ) {
        route.push_operation(op).unwrap();
        assert_eq!(route, expected)
    }

    #[rstest]
    fn can_redo_operation(
        #[from(after_undo)] mut route: Route,
        #[from(full_route)] expected: Route,
    ) {
        route.redo_operation().unwrap();
        assert_eq!(route, expected)
    }

    #[rstest]
    #[case::empty(empty_route())]
    #[case::full(full_route())]
    fn cannot_redo_if_nothing_to_redo(#[case] mut route: Route) {
        assert!(matches!(
            route.redo_operation(),
            // TODO: This might be an DomainError
            // maybe this should be catched at UseCase
            // and then converted into InvalidOperation
            // => https://github.com/team-azb/route-bucket-backend/issues/81
            Err(ApplicationError::InvalidOperation(_))
        ))
    }

    #[rstest]
    fn can_undo_operation(
        #[from(full_route)] mut route: Route,
        #[from(after_undo)] expected: Route,
    ) {
        route.undo_operation().unwrap();
        assert_eq!(route, expected)
    }

    #[rstest]
    fn cannot_undo_if_empty(mut empty_route: Route) {
        assert!(matches!(
            empty_route.redo_operation(),
            Err(ApplicationError::InvalidOperation(_))
        ))
    }

    #[rstest]
    fn can_calc_route_features_from_seg_list(
        #[from(full_route_filled_but_yet_to_calc_features)] mut route: Route,
        #[from(full_route_filled)] expected: Route,
    ) {
        route.calc_route_features_from_seg_list().unwrap();
        assert_eq!(route, expected)
    }

    macro_rules! init_empty_route {
        ($op_num:expr, $op_list_name:ident, $seg_list_name:ident) => {
            Route {
                info: RouteInfo::empty_route0($op_num),
                op_list: Operation::$op_list_name(),
                seg_list: SegmentList::$seg_list_name(false, false, true),
            }
        };
    }

    pub trait RouteFixtures {
        fn empty() -> Route {
            Route {
                info: RouteInfo::empty_route0(0),
                op_list: Vec::new(),
                seg_list: SegmentList::empty(),
            }
        }

        fn yokohama() -> Route {
            init_empty_route!(1, after_add_yokohama_op_list, yokohama)
        }

        fn yokohama_to_chiba() -> Route {
            init_empty_route!(2, after_add_chiba_op_list, yokohama_to_chiba)
        }

        fn yokohama_to_chiba_via_tokyo() -> Route {
            init_empty_route!(3, after_add_tokyo_op_list, yokohama_to_chiba_via_tokyo)
        }

        fn yokohama_to_chiba_after_remove() -> Route {
            init_empty_route!(4, after_remove_tokyo_op_list, yokohama_to_chiba)
        }

        fn yokohama_to_chiba_after_undo() -> Route {
            init_empty_route!(2, after_add_tokyo_op_list, yokohama_to_chiba)
        }

        fn yokohama_to_tokyo() -> Route {
            init_empty_route!(3, after_move_chiba_op_list, yokohama_to_tokyo)
        }

        fn yokohama_to_chiba_filled(set_ele: bool, set_features: bool) -> Route {
            Route::new(
                if set_features {
                    RouteInfo::yokohama_to_chiba()
                } else {
                    RouteInfo::empty_route0(2)
                },
                Operation::after_add_tokyo_op_list(),
                SegmentList::yokohama_to_chiba(set_ele, set_features, false),
            )
        }

        fn yokohama_to_chiba_via_tokyo_filled(set_ele: bool, set_features: bool) -> Route {
            Route::new(
                if set_features {
                    RouteInfo::yokohama_to_chiba_via_tokyo()
                } else {
                    RouteInfo::empty_route0(3)
                },
                Operation::after_add_tokyo_op_list(),
                SegmentList::yokohama_to_chiba_via_tokyo(set_ele, set_features, false),
            )
        }

        fn yokohama_to_tokyo_filled(set_ele: bool, set_features: bool) -> Route {
            Route::new(
                if set_features {
                    RouteInfo::yokohama_to_tokyo()
                } else {
                    RouteInfo::empty_route0(3)
                },
                Operation::after_move_chiba_op_list(),
                SegmentList::yokohama_to_tokyo(set_ele, set_features, false),
            )
        }
    }

    impl RouteFixtures for Route {}
}
