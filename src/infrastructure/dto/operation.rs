use crate::domain::operation_history::Operation;
use crate::domain::polyline::Polyline;
use crate::domain::types::RouteId;
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
    pos: Option<u32>,
    polyline: String,
}

impl OperationDto {
    pub fn to_model(&self) -> ApplicationResult<Operation> {
        Operation::from_code(&self.code, self.pos, &self.polyline)
    }

    pub fn from_model(
        operation: &Operation,
        route_id: &RouteId,
        index: u32,
    ) -> ApplicationResult<OperationDto> {
        let pos_op: Option<u32>;
        let polyline: Polyline;
        match operation {
            Operation::Add { pos, coord } | Operation::Remove { pos, coord } => {
                pos_op = Some(*pos);
                polyline = Polyline::from_vec(vec![coord.clone()]);
            }
            Operation::Clear { org_list: list } | Operation::InitWithList { list } => {
                pos_op = None;
                polyline = list.clone();
            }
        };
        Ok(OperationDto {
            route_id: route_id.to_string(),
            index,
            code: operation.to_code().to_string(),
            pos: pos_op,
            polyline: polyline.encode()?,
        })
    }
}
