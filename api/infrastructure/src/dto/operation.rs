use getset::Getters;
use route_bucket_domain::model::{Coordinate, Operation, OperationType, Polyline, RouteId};
use route_bucket_utils::ApplicationResult;
use std::convert::TryFrom;

/// 座標のdto構造体
#[derive(sqlx::FromRow, Getters)]
#[get = "pub"]
pub struct OperationDto {
    route_id: String,
    index: u32,
    code: String,
    pos: u32,
    polyline: String,
}

impl OperationDto {
    pub fn into_model(self) -> ApplicationResult<Operation> {
        let OperationDto {
            code,
            pos,
            polyline,
            ..
        } = self;
        let op_type = OperationType::try_from(code)?;

        let [org_coords, new_coords] = <[Vec<Coordinate>; 2]>::try_from(
            polyline
                .split(" ")
                .map(String::from)
                .map(Polyline::from)
                .map(Vec::try_from)
                .collect::<ApplicationResult<Vec<_>>>()?,
        )
        .unwrap();

        Ok(Operation::new(
            op_type,
            pos as usize,
            org_coords,
            new_coords,
        ))
    }

    pub fn from_model(
        operation: &Operation,
        route_id: &RouteId,
        index: u32,
    ) -> ApplicationResult<OperationDto> {
        let org_polyline: String = Polyline::from(operation.org_coords().clone()).into();
        let new_polyline: String = Polyline::from(operation.new_coords().clone()).into();

        Ok(OperationDto {
            route_id: route_id.to_string(),
            index,
            code: operation.op_type().clone().into(),
            pos: *operation.start_pos() as u32,
            polyline: [org_polyline, new_polyline].join(" "),
        })
    }
}
