use std::convert::{TryFrom, TryInto};

use itertools::Itertools;

use crate::domain::model::linestring::Coordinate;
use crate::domain::model::operation::{Operation, OperationStruct, OperationType};
use crate::domain::model::types::{Polyline, RouteId};
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::schema::operations;
use crate::utils::error::ApplicationResult;

/// 座標のdto構造体
#[derive(Identifiable, Queryable, Insertable, Associations, Debug)]
#[table_name = "operations"]
#[primary_key(route_id, index)]
#[belongs_to(RouteDto, foreign_key = "route_id")]
pub struct OperationDto {
    route_id: String,
    index: u32,
    code: String,
    pos: u32,
    polyline: String,
}

impl OperationDto {
    pub fn to_model(&self) -> ApplicationResult<Operation> {
        let op_type = OperationType::try_from(self.code.clone())?;

        let [org_coords, new_coords] = <[Vec<Coordinate>; 2]>::try_from(
            self.polyline
                .clone()
                .split(" ")
                .map(String::from)
                .map(Polyline::from)
                .map(Vec::try_from)
                .try_collect()?,
        )
        .unwrap();

        Ok(Operation::new(
            op_type,
            *self.pos as usize,
            org_coords,
            new_coords,
        ))
    }

    pub fn from_model(
        operation: &Operation,
        route_id: &RouteId,
        index: u32,
    ) -> ApplicationResult<OperationDto> {
        let org_polyline: String = operation.org_coords().clone().into();
        let new_polyline: String = operation.new_coords().clone().into();

        Ok(OperationDto {
            route_id: route_id.to_string(),
            index,
            code: operation.op_type().into(),
            pos: operation.start_pos() as u32,
            polyline: [org_polyline, new_polyline].join(" "),
        })
    }
}
