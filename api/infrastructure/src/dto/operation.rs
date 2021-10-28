use getset::Getters;
use itertools::Itertools;
use route_bucket_domain::model::{
    DrawingMode, Operation, OperationId, OperationType, Polyline, RouteId, SegmentTemplate,
};
use route_bucket_utils::{ApplicationError, ApplicationResult};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SegmentTemplateDto {
    start: String,
    goal: String,
    mode: String,
}

impl From<SegmentTemplate> for SegmentTemplateDto {
    fn from(template: SegmentTemplate) -> Self {
        let (start, goal, mode) = template.into();
        Self {
            start: Polyline::from(start).into(),
            goal: Polyline::from(goal).into(),
            mode: mode.to_string(),
        }
    }
}

impl TryFrom<SegmentTemplateDto> for SegmentTemplate {
    type Error = ApplicationError;

    fn try_from(dto: SegmentTemplateDto) -> ApplicationResult<Self> {
        Ok(Self::new(
            Polyline::from(dto.start).try_into()?,
            Polyline::from(dto.goal).try_into()?,
            DrawingMode::from_str(&dto.mode).map_err(|err| {
                ApplicationError::DataBaseError(format!("Invalid mode found in DB: {:?}", err))
            })?,
        ))
    }
}

/// 座標のdto構造体
#[derive(sqlx::FromRow, Getters)]
#[get = "pub"]
pub struct OperationDto {
    id: String,
    route_id: String,
    index: u32,
    code: String,
    pos: u32,
    org_seg_templates: sqlx::types::Json<Vec<SegmentTemplateDto>>,
    new_seg_templates: sqlx::types::Json<Vec<SegmentTemplateDto>>,
}

impl OperationDto {
    pub fn into_model(self) -> ApplicationResult<Operation> {
        let OperationDto {
            id,
            code,
            pos,
            org_seg_templates,
            new_seg_templates,
            ..
        } = self;
        let op_type = OperationType::from_str(&code)
            .map_err(|_| ApplicationError::DomainError(format!("Invalid type code: {}", code)))?;

        Ok(Operation::from((
            OperationId::from_string(id),
            op_type,
            pos as usize,
            org_seg_templates
                .0
                .into_iter()
                .map(SegmentTemplate::try_from)
                .try_collect()?,
            new_seg_templates
                .0
                .into_iter()
                .map(SegmentTemplate::try_from)
                .try_collect()?,
        )))
    }

    pub fn from_model(
        operation: &Operation,
        route_id: &RouteId,
        index: u32,
    ) -> ApplicationResult<OperationDto> {
        Ok(OperationDto {
            id: operation.id().to_string(),
            route_id: route_id.to_string(),
            index,
            code: operation.op_type().to_string(),
            pos: *operation.pos() as u32,
            org_seg_templates: sqlx::types::Json(
                operation
                    .org_seg_templates()
                    .clone()
                    .into_iter()
                    .map(SegmentTemplateDto::from)
                    .collect(),
            ),
            new_seg_templates: sqlx::types::Json(
                operation
                    .new_seg_templates()
                    .clone()
                    .into_iter()
                    .map(SegmentTemplateDto::from)
                    .collect(),
            ),
        })
    }
}
