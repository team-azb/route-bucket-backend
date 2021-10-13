use std::convert::TryFrom;
use std::mem::swap;

use getset::Getters;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::types::OperationId;

use super::coordinate::Coordinate;
use super::segment_list::SegmentList;

#[cfg(test)]
use derivative::Derivative;

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
#[cfg_attr(test, derive(Derivative))]
#[cfg_attr(test, derivative(PartialEq))]
pub struct Operation {
    #[cfg_attr(test, derivative(PartialEq = "ignore"))]
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
    use rstest::{fixture, rstest};

    use crate::model::route::coordinate::tests::CoordinateFixtures;
    use crate::model::route::segment_list::tests::SegmentListFixture;

    use super::*;

    #[fixture]
    fn add_tokyo() -> Operation {
        Operation::add_tokyo()
    }

    #[fixture]
    fn remove_tokyo() -> Operation {
        Operation::remove_tokyo()
    }

    #[fixture]
    fn move_chiba_to_tokyo() -> Operation {
        Operation::move_chiba_to_tokyo()
    }

    #[fixture]
    fn move_tokyo_to_chiba() -> Operation {
        Operation::new_move(
            1,
            Coordinate::chiba(false, None),
            Coordinate::yokohama_to_tokyo_coords(false, None),
        )
    }

    #[rstest]
    #[case::add(add_tokyo(), remove_tokyo())]
    #[case::remove(remove_tokyo(), add_tokyo())]
    #[case::move_(move_tokyo_to_chiba(), move_chiba_to_tokyo())]
    fn can_reverse_to_inverse_operation(#[case] mut op: Operation, #[case] op_inv: Operation) {
        op.reverse();
        assert_eq!(op, op_inv)
    }

    #[rstest]
    #[case::add(
        add_tokyo(),
        SegmentList::yokohama_to_chiba(false, false, true),
        SegmentList::yokohama_to_chiba_via_tokyo(false, false, true)
    )]
    #[case::remove(
        remove_tokyo(),
        SegmentList::yokohama_to_chiba_via_tokyo(false, false, true),
        SegmentList::yokohama_to_chiba(false, false, true)
    )]
    #[case::move_(
        move_chiba_to_tokyo(),
        SegmentList::yokohama_to_chiba(false, false, true),
        SegmentList::yokohama_to_tokyo(false, false, true)
    )]
    fn can_apply_to_seg_list(
        #[case] op: Operation,
        #[case] mut seg_list: SegmentList,
        #[case] expected: SegmentList,
    ) {
        op.apply(&mut seg_list).unwrap();
        assert_eq!(seg_list, expected)
    }

    macro_rules! concat_op_list {
        ($op_list_name:ident, $op_name:ident) => {
            vec![Operation::$op_list_name(), vec![Operation::$op_name()]].concat()
        };
    }
    pub trait OperationFixtures {
        fn add_yokohama() -> Operation {
            Operation::new_add(0, Coordinate::yokohama(false, None))
        }

        fn add_chiba() -> Operation {
            Operation::new_add(1, Coordinate::chiba(false, None))
        }

        fn add_tokyo() -> Operation {
            Operation::new_add(1, Coordinate::tokyo(false, None))
        }

        fn remove_tokyo() -> Operation {
            Operation::new_remove(
                1,
                Coordinate::yokohama_to_chiba_via_tokyo_coords(false, None),
            )
        }

        fn move_chiba_to_tokyo() -> Operation {
            Operation::new_move(
                1,
                Coordinate::tokyo(false, None),
                Coordinate::yokohama_to_chiba_coords(false, None),
            )
        }

        fn after_add_yokohama_op_list() -> Vec<Operation> {
            vec![Self::add_yokohama()]
        }

        fn after_add_chiba_op_list() -> Vec<Operation> {
            concat_op_list!(after_add_yokohama_op_list, add_chiba)
        }

        fn after_add_tokyo_op_list() -> Vec<Operation> {
            concat_op_list!(after_add_chiba_op_list, add_tokyo)
        }

        fn after_remove_tokyo_op_list() -> Vec<Operation> {
            concat_op_list!(after_add_tokyo_op_list, remove_tokyo)
        }

        fn after_move_chiba_op_list() -> Vec<Operation> {
            concat_op_list!(after_add_chiba_op_list, move_chiba_to_tokyo)
        }
    }

    impl OperationFixtures for Operation {}
}
