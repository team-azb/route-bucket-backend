use std::convert::TryFrom;
use std::mem::swap;

use getset::Getters;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::types::OperationId;

use super::coordinate::Coordinate;
use super::segment_list::SegmentList;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationType {
    Add,
    Remove,
    Move,
}

impl OperationType {
    pub fn reverse(&self) -> Self {
        match *self {
            OperationType::Add => OperationType::Remove,
            OperationType::Remove => OperationType::Add,
            OperationType::Move => OperationType::Move,
        }
    }
}

impl TryFrom<String> for OperationType {
    type Error = ApplicationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "ad" => Ok(OperationType::Add),
            "rm" => Ok(OperationType::Remove),
            "mv" => Ok(OperationType::Move),
            _ => Err(ApplicationError::DomainError(format!(
                "Cannot convert {} into OperationType!",
                value
            ))),
        }
    }
}

impl From<OperationType> for String {
    fn from(value: OperationType) -> Self {
        match value {
            OperationType::Add => "ad",
            OperationType::Remove => "rm",
            OperationType::Move => "mv",
        }
        .into()
    }
}

#[derive(Clone, Debug, Getters)]
#[get = "pub"]
pub struct Operation {
    id: OperationId,
    op_type: OperationType,
    pos: usize,
    org_coord: Option<Coordinate>,
    new_coord: Option<Coordinate>,
}

impl Operation {
    pub fn new(
        op_type: OperationType,
        pos: usize,
        org_coord: Option<Coordinate>,
        new_coord: Option<Coordinate>,
    ) -> Self {
        Self {
            id: OperationId::new(),
            op_type,
            pos,
            org_coord,
            new_coord,
        }
    }

    pub fn new_add(pos: usize, coord: Coordinate) -> Self {
        Self::new(OperationType::Add, pos, None, Some(coord))
    }

    pub fn new_remove(pos: usize, org_waypoints: Vec<Coordinate>) -> Self {
        let org = org_waypoints[pos].clone();
        Self::new(OperationType::Remove, pos, Some(org), None)
    }

    pub fn new_move(pos: usize, coord: Coordinate, org_waypoints: Vec<Coordinate>) -> Self {
        let org = org_waypoints[pos].clone();
        Self::new(OperationType::Move, pos, Some(org), Some(coord))
    }

    pub fn apply(&self, seg_list: &mut SegmentList) -> ApplicationResult<()> {
        match self.op_type {
            OperationType::Remove => seg_list.remove_waypoint(self.pos),
            OperationType::Add | OperationType::Move => {
                if let Some(new_coord) = self.new_coord.clone() {
                    if self.op_type == OperationType::Add {
                        seg_list.insert_waypoint(self.pos, new_coord)
                    } else {
                        seg_list.move_waypoint(self.pos, new_coord)
                    }
                } else {
                    Err(ApplicationError::DomainError(
                        "OperationType::{Add | Move} must have new_coord!".into(),
                    ))
                }
            }
        }
    }

    pub fn reverse(&mut self) {
        self.op_type = self.op_type.reverse();
        swap(&mut self.org_coord, &mut self.new_coord);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use rstest::rstest;

    use crate::model::route::coordinate::tests::CoordinateFixtures;
    use crate::model::route::segment_list::tests::SegmentListFixture;

    use super::*;

    pub trait OperationFixtures {
        fn step1_add_yokohama() -> Operation {
            Operation::new_add(0, Coordinate::yokohama())
        }

        fn step2_add_chiba() -> Operation {
            Operation::new_add(1, Coordinate::chiba())
        }

        fn step3_add_tokyo() -> Operation {
            Operation::new_add(1, Coordinate::tokyo())
        }

        fn step4_rm_tokyo() -> Operation {
            Operation::new_remove(1, Coordinate::yokohama_to_chiba_waypoints())
        }

        fn step5_rm_chiba() -> Operation {
            Operation::new_remove(1, vec![Coordinate::yokohama(), Coordinate::chiba()])
        }

        fn step6_rm_yokohama() -> Operation {
            Operation::new_remove(0, vec![Coordinate::yokohama()])
        }

        fn step7_init() -> Operation {
            Operation::new(
                OperationType::InitWithList,
                0,
                vec![],
                vec![Coordinate::yokohama(), Coordinate::chiba()],
            )
        }

        fn step8_mv_to_tokyo() -> Operation {
            Operation::new_move(
                1,
                Coordinate::tokyo(),
                vec![Coordinate::yokohama(), Coordinate::chiba()],
            )
        }

        fn step9_mv_to_chiba() -> Operation {
            Operation::new_move(
                1,
                Coordinate::chiba(),
                vec![Coordinate::yokohama(), Coordinate::tokyo()],
            )
        }

        fn step10_clear() -> Operation {
            Operation::new_clear(vec![Coordinate::yokohama(), Coordinate::chiba()])
        }
    }

    impl OperationFixtures for Operation {}

    #[rstest]
    #[case::add_front(Operation::step1_add_yokohama(), Operation::step6_rm_yokohama())]
    #[case::add_back(Operation::step2_add_chiba(), Operation::step5_rm_chiba())]
    #[case::add_middle(Operation::step3_add_tokyo(), Operation::step4_rm_tokyo())]
    #[case::rm_middle(Operation::step4_rm_tokyo(), Operation::step3_add_tokyo())]
    #[case::rm_back(Operation::step5_rm_chiba(), Operation::step2_add_chiba())]
    #[case::rm_front(Operation::step6_rm_yokohama(), Operation::step1_add_yokohama())]
    #[case::init(Operation::step7_init(), Operation::step10_clear())]
    #[case::mv(Operation::step8_mv_to_tokyo(), Operation::step9_mv_to_chiba())]
    #[case::mv(Operation::step9_mv_to_chiba(), Operation::step8_mv_to_tokyo())]
    #[case::clear(Operation::step10_clear(), Operation::step7_init())]
    fn can_reverse_to_inverse_operation(#[case] mut op: Operation, #[case] op_inv: Operation) {
        op.reverse();
        assert_eq!(op, op_inv)
    }

    #[rstest]
    #[case::add_front(
        Operation::step1_add_yokohama(),
        SegmentList::step0_empty(),
        SegmentList::step1_add_yokohama(false, false, true)
    )]
    #[case::add_back(
        Operation::step2_add_chiba(),
        SegmentList::step1_add_yokohama(false, false, false),
        SegmentList::step2_add_chiba(false, false, true)
    )]
    #[case::add_middle(
        Operation::step3_add_tokyo(),
        SegmentList::step2_add_chiba(false, false, false),
        SegmentList::step3_add_tokyo(false, false, true)
    )]
    // #[case::rm_middle(Operation::step4_rm_tokyo(), Operation::step3_add_tokyo())]
    // #[case::rm_back(Operation::step5_rm_chiba(), Operation::step2_add_chiba())]
    // #[case::rm_front(Operation::step6_rm_yokohama(), Operation::step1_add_yokohama())]
    // #[case::init(Operation::step7_init(), Operation::step10_clear())]
    // #[case::mv(Operation::step8_mv_to_tokyo(), Operation::step9_mv_to_chiba())]
    // #[case::mv(Operation::step9_mv_to_chiba(), Operation::step8_mv_to_tokyo())]
    // #[case::clear(Operation::step10_clear(), Operation::step7_init())]
    fn can_apply_to_seg_list(
        #[case] op: Operation,
        #[case] mut seg_list: SegmentList,
        #[case] expected: SegmentList,
    ) {
        op.apply(&mut seg_list).unwrap();
        assert_eq!(seg_list, expected)
    }
}
