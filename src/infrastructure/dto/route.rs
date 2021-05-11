use crate::domain::model::polyline::Polyline;
use crate::domain::model::route::Route;
use crate::domain::model::types::RouteId;
use crate::infrastructure::schema::routes;
use crate::utils::error::ApplicationResult;

/// ルートのdto構造体
#[derive(Identifiable, Queryable, Insertable, Debug, AsChangeset)]
#[table_name = "routes"]
pub struct RouteDto {
    id: String,
    name: String,
    polyline: String,
    operation_pos: u32,
}

impl RouteDto {
    pub fn to_model(&self) -> ApplicationResult<Route> {
        Ok(Route::new(
            RouteId::from_string(self.id.clone()),
            &self.name,
            Polyline::decode(&self.polyline)?,
            self.operation_pos as usize,
        ))
    }

    pub fn from_model(route: &Route) -> ApplicationResult<RouteDto> {
        Ok(RouteDto {
            id: route.id().to_string(),
            name: route.name().clone(),
            polyline: route.polyline().encode()?,
            operation_pos: *route.op_num() as u32,
        })
    }
}
