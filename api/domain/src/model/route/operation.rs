use std::mem::swap;

use derive_more::{From, Into};
use getset::Getters;
use serde::{Deserialize, Serialize};

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::types::NanoId;

use super::coordinate::Coordinate;
use super::segment_list::{DrawingMode, Segment, SegmentList};

#[cfg(any(test, feature = "fixtures"))]
use derivative::Derivative;

pub type OperationId = NanoId<Operation, 21>;

#[derive(Clone, Debug, PartialEq, Eq, strum::Display, strum::EnumString)]
pub enum OperationType {
    #[strum(serialize = "ad")]
    Add,
    #[strum(serialize = "rm")]
    Remove,
    #[strum(serialize = "mv")]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, From, Into)]
pub struct SegmentTemplate {
    start: Coordinate,
    goal: Coordinate,
    mode: DrawingMode,
}

impl SegmentTemplate {
    pub fn new(start: Coordinate, goal: Coordinate, mode: DrawingMode) -> Self {
        Self { start, goal, mode }
    }

    pub fn from_segment(segment: &Segment) -> Self {
        Self::new(
            segment.start().clone(),
            segment.goal().clone(),
            *segment.mode(),
        )
    }
}

impl From<SegmentTemplate> for Segment {
    fn from(template: SegmentTemplate) -> Self {
        Segment::new_empty(template.start, template.goal, template.mode)
    }
}

#[derive(Clone, Debug, Getters, From)]
#[get = "pub"]
#[cfg_attr(any(test, feature = "fixtures"), derive(Derivative))]
#[cfg_attr(any(test, feature = "fixtures"), derivative(PartialEq))]
pub struct Operation {
    #[cfg_attr(any(test, feature = "fixtures"), derivative(PartialEq = "ignore"))]
    id: OperationId,
    // NOTE: typeはロジック的には不要だが、デバッグ用に残している
    op_type: OperationType,
    pos: usize,
    org_seg_templates: Vec<SegmentTemplate>,
    new_seg_templates: Vec<SegmentTemplate>,
}

impl Operation {
    pub fn new(
        op_type: OperationType,
        pos: usize,
        org_seg_templates: Vec<SegmentTemplate>,
        new_seg_templates: Vec<SegmentTemplate>,
    ) -> Self {
        Self {
            id: OperationId::new(),
            op_type,
            pos,
            org_seg_templates,
            new_seg_templates,
        }
    }

    pub fn new_add(
        pos: usize,
        coord: Coordinate,
        org_seg_list: &SegmentList,
        mode: DrawingMode,
    ) -> ApplicationResult<Self> {
        let mut org_seg_templates = Vec::new();
        let mut new_seg_templates = Vec::new();

        if pos <= org_seg_list.len() {
            if pos > 0 {
                let org_seg = &org_seg_list.segments[pos - 1];
                let start = org_seg.start();
                org_seg_templates = vec![SegmentTemplate::from_segment(org_seg)];
                new_seg_templates = vec![SegmentTemplate::new(start.clone(), coord.clone(), mode)];
            }

            let goal = org_seg_list
                .segments
                .get(pos)
                .map(Segment::start)
                .unwrap_or_else(|| &coord);

            new_seg_templates.push(SegmentTemplate::new(coord.clone(), goal.clone(), mode));
        } else {
            return Err(ApplicationError::DomainError(format!(
                "pos({}) cannot be greater than org_seg_list.len()({}) at Operation::new_add",
                pos,
                org_seg_list.len()
            )));
        }

        Ok(Self::new(
            OperationType::Add,
            pos.saturating_sub(1),
            org_seg_templates,
            new_seg_templates,
        ))
    }

    pub fn new_remove(
        pos: usize,
        org_seg_list: &SegmentList,
        mode: DrawingMode,
    ) -> ApplicationResult<Self> {
        let mut org_seg_templates = Vec::new();
        let mut new_seg_templates = Vec::new();

        if pos < org_seg_list.len() {
            if pos > 0 {
                let start = org_seg_list.segments[pos - 1].start();
                let goal = org_seg_list.segments[pos].goal();
                org_seg_templates = vec![SegmentTemplate::from_segment(
                    &org_seg_list.segments[pos - 1],
                )];
                new_seg_templates = vec![SegmentTemplate::new(start.clone(), goal.clone(), mode)];
            }

            org_seg_templates.push(SegmentTemplate::from_segment(&org_seg_list.segments[pos]));
        } else {
            return Err(ApplicationError::DomainError(format!(
                "pos({}) cannot be greater than or equal to org_seg_list.len()({}) at Operation::new_remove",
                pos,
                org_seg_list.len()
            )));
        }

        Ok(Self::new(
            OperationType::Remove,
            pos.saturating_sub(1),
            org_seg_templates,
            new_seg_templates,
        ))
    }

    pub fn new_move(
        pos: usize,
        coord: Coordinate,
        org_seg_list: &SegmentList,
        mode: DrawingMode,
    ) -> ApplicationResult<Self> {
        let mut org_seg_templates = Vec::new();
        let mut new_seg_templates = Vec::new();

        if pos < org_seg_list.len() {
            if pos > 0 {
                let start = org_seg_list.segments[pos - 1].start();
                org_seg_templates = vec![SegmentTemplate::from_segment(
                    &org_seg_list.segments[pos - 1],
                )];
                new_seg_templates = vec![SegmentTemplate::new(start.clone(), coord.clone(), mode)];
            }

            let goal = org_seg_list
                .segments
                .get(pos + 1)
                .map(Segment::start)
                .unwrap_or_else(|| &coord);

            org_seg_templates.push(SegmentTemplate::from_segment(&org_seg_list.segments[pos]));
            new_seg_templates.push(SegmentTemplate::new(coord.clone(), goal.clone(), mode));
        } else {
            return Err(ApplicationError::DomainError(format!(
                "pos({}) cannot be greater than or equal to org_seg_list.len()({}) at Operation::new_move",
                pos,
                org_seg_list.len()
            )));
        }

        Ok(Self::new(
            OperationType::Move,
            pos.saturating_sub(1),
            org_seg_templates,
            new_seg_templates,
        ))
    }

    pub fn apply(&self, seg_list: &mut SegmentList) -> ApplicationResult<()> {
        seg_list.splice(
            self.pos..self.pos + self.org_seg_templates.len(),
            self.new_seg_templates
                .iter()
                .map(Clone::clone)
                .map(Segment::from),
        );
        Ok(())
    }

    pub fn reverse(&mut self) {
        self.op_type = self.op_type.reverse();
        swap(&mut self.org_seg_templates, &mut self.new_seg_templates);
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod tests {
    use rstest::{fixture, rstest};

    use crate::model::route::coordinate::tests::CoordinateFixtures;

    #[cfg(test)]
    use crate::model::route::segment_list::tests::SegmentListFixture;

    use super::*;

    macro_rules! init_template {
        ($start:ident, $goal:ident, $mode:ident) => {
            SegmentTemplate::new(
                Coordinate::$start(false, None),
                Coordinate::$goal(false, None),
                DrawingMode::$mode,
            )
        };
    }

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
        Operation {
            id: OperationId::new(),
            op_type: OperationType::Move,
            pos: 0,
            org_seg_templates: vec![
                init_template!(yokohama, tokyo, Freehand),
                init_template!(tokyo, tokyo, Freehand),
            ],
            new_seg_templates: vec![
                init_template!(yokohama, chiba, FollowRoad),
                init_template!(chiba, chiba, FollowRoad),
            ],
        }
    }

    #[rstest]
    fn can_new_add() {
        assert_eq!(
            Operation::new_add(
                1,
                Coordinate::tokyo(false, None),
                &SegmentList::yokohama_to_chiba(false, false, true),
                DrawingMode::Freehand,
            ),
            Ok(add_tokyo())
        )
    }

    #[rstest]
    fn can_new_remove() {
        assert_eq!(
            Operation::new_remove(
                1,
                &SegmentList::yokohama_to_chiba_via_tokyo(false, false, true),
                DrawingMode::FollowRoad,
            ),
            Ok(remove_tokyo())
        )
    }

    #[rstest]
    fn can_new_move() {
        assert_eq!(
            Operation::new_move(
                1,
                Coordinate::tokyo(false, None),
                &SegmentList::yokohama_to_chiba(false, false, true),
                DrawingMode::Freehand,
            ),
            Ok(move_chiba_to_tokyo())
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
            Operation {
                id: OperationId::new(),
                op_type: OperationType::Add,
                pos: 0,
                org_seg_templates: vec![],
                new_seg_templates: vec![init_template!(yokohama, yokohama, FollowRoad)],
            }
        }

        fn add_chiba() -> Operation {
            Operation {
                id: OperationId::new(),
                op_type: OperationType::Add,
                pos: 0,
                org_seg_templates: vec![init_template!(yokohama, yokohama, FollowRoad)],
                new_seg_templates: vec![
                    init_template!(yokohama, chiba, FollowRoad),
                    init_template!(chiba, chiba, FollowRoad),
                ],
            }
        }

        fn add_tokyo() -> Operation {
            Operation {
                id: OperationId::new(),
                op_type: OperationType::Add,
                pos: 0,
                org_seg_templates: vec![init_template!(yokohama, chiba, FollowRoad)],
                new_seg_templates: vec![
                    init_template!(yokohama, tokyo, Freehand),
                    init_template!(tokyo, chiba, Freehand),
                ],
            }
        }

        fn remove_tokyo() -> Operation {
            Operation {
                id: OperationId::new(),
                op_type: OperationType::Remove,
                pos: 0,
                org_seg_templates: vec![
                    init_template!(yokohama, tokyo, Freehand),
                    init_template!(tokyo, chiba, Freehand),
                ],
                new_seg_templates: vec![init_template!(yokohama, chiba, FollowRoad)],
            }
        }

        fn move_chiba_to_tokyo() -> Operation {
            Operation {
                id: OperationId::new(),
                op_type: OperationType::Move,
                pos: 0,
                org_seg_templates: vec![
                    init_template!(yokohama, chiba, FollowRoad),
                    init_template!(chiba, chiba, FollowRoad),
                ],
                new_seg_templates: vec![
                    init_template!(yokohama, tokyo, Freehand),
                    init_template!(tokyo, tokyo, Freehand),
                ],
            }
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
