use getset::Getters;
use route_bucket_domain::model::{Coordinate, Operation, OperationType, Polyline, RouteId};
use route_bucket_utils::ApplicationResult;
use std::convert::TryFrom;

/// 座標のdto構造体
#[derive(sqlx::FromRow, Getters)]
#[get = "pub"]
pub struct OperationDto {
    id: String,
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

        let [org_coord, new_coord] = <[Option<Coordinate>; 2]>::try_from(
            polyline
                .split(" ")
                .map(String::from)
                .map(Polyline::from)
                .map(Option::try_from)
                .collect::<ApplicationResult<Vec<_>>>()?,
        )
        .unwrap();

        Ok(Operation::new(op_type, pos as usize, org_coord, new_coord))
    }

    pub fn from_model(
        operation: &Operation,
        route_id: &RouteId,
        index: u32,
    ) -> ApplicationResult<OperationDto> {
        let org_polyline: String = Polyline::from(operation.org_coord().clone()).into();
        let new_polyline: String = Polyline::from(operation.new_coord().clone()).into();

        Ok(OperationDto {
            id: operation.id().to_string(),
            route_id: route_id.to_string(),
            index,
            code: operation.op_type().clone().into(),
            pos: *operation.pos() as u32,
            polyline: [org_polyline, new_polyline].join(" "),
        })
    }
}
