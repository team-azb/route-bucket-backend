use crate::domain::operation_history::OperationHistory;
use crate::domain::polyline::Polyline;
use crate::domain::route::Route;
use crate::domain::types::RouteId;
use crate::infrastructure::dto::operation::OperationDto;
use crate::infrastructure::schema::routes;
use crate::utils::error::ApplicationResult;

/// ルートのdto構造体
#[derive(Identifiable, Queryable, Insertable, Debug)]
#[table_name = "routes"]
pub struct RouteDto {
    id: String,
    name: String,
    polyline: String,
    operation_pos: u32,
}

impl RouteDto {
    pub fn to_model(&self, op_dtos: Vec<OperationDto>) -> ApplicationResult<Route> {
        let operations = op_dtos
            .iter()
            .map(OperationDto::to_model)
            .collect::<ApplicationResult<Vec<_>>>()?;
        Ok(Route::new(
            RouteId::from_string(self.id.clone()),
            &self.name,
            Polyline::decode(&self.polyline)?,
            OperationHistory::new(operations, self.operation_pos as usize),
        ))
    }

    pub fn from_model(route: &Route) -> ApplicationResult<RouteDto> {
        Ok(RouteDto {
            id: route.id().to_string(),
            name: route.name().clone(),
            polyline: route.polyline().encode()?,
            operation_pos: *route.operation_history().pos() as u32,
        })
    }
}
