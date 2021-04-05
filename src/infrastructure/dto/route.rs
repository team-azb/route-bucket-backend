use crate::domain::polyline::Polyline;
use crate::domain::route::Route;
use crate::domain::types::RouteId;
use crate::infrastructure::dto::coordinate::CoordinateDto;
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
    pub fn to_model(&self, point_dtos: Vec<CoordinateDto>) -> ApplicationResult<Route> {
        let points = point_dtos
            .iter()
            .map(CoordinateDto::to_model)
            .collect::<ApplicationResult<Vec<_>>>()?;

        Ok(Route::new(
            RouteId::from_string(self.id.clone()),
            &self.name,
            Polyline::decode(&self.polyline)?,
            OperationHistory::new(operations, self.operation_pos as usize),
        ))
    }

    pub fn from_model(route: &Route) -> RouteDto {
        RouteDto {
            id: route.id().to_string(),
            name: route.name().clone(),
            polyline: route.polyline().encode()?,
            operation_pos: *route.operation_history().pos() as u32,
        })
    }
}
