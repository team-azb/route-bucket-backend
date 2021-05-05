use crate::domain::model::operation::{Operation, OperationStruct};
use crate::domain::model::polyline::Polyline;
use crate::domain::model::types::RouteId;
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::schema::operations;
use crate::utils::error::ApplicationResult;
use std::convert::{TryFrom, TryInto};

/// 座標のdto構造体
#[derive(Identifiable, Queryable, Insertable, Associations, Debug)]
#[table_name = "operations"]
#[primary_key(route_id, index)]
#[belongs_to(RouteDto, foreign_key = "route_id")]
pub struct OperationDto {
    route_id: String,
    index: u32,
    code: String,
    pos: Option<u32>,
    polyline: String,
}

impl OperationDto {
    pub fn to_model(&self) -> ApplicationResult<Operation> {
        OperationStruct::new(
            self.code.clone(),
            self.pos.map(|u| u as usize),
            None,
            None,
            Some(Polyline::decode(&self.polyline)?),
        )
        .map(OperationStruct::try_into)?
    }

    pub fn from_model(
        operation: &Operation,
        route_id: &RouteId,
        index: u32,
    ) -> ApplicationResult<OperationDto> {
        let opst = OperationStruct::try_from(operation.clone())?;
        Ok(OperationDto {
            route_id: route_id.to_string(),
            index,
            code: opst.code().clone(),
            pos: opst.pos().clone().map(|u| u as u32),
            polyline: opst.polyline().encode()?,
        })
    }
}
