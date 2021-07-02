use crate::domain::model::route::Route;
use crate::domain::model::types::{Polyline, RouteId};
use crate::infrastructure::schema::routes;
use crate::utils::error::ApplicationResult;
use std::convert::TryInto;

/// ルートのdto構造体
#[derive(Identifiable, Queryable, Insertable, Debug, AsChangeset)]
#[table_name = "routes"]
pub struct RouteDto {
    id: String,
    name: String,
    waypoint_polyline: String,
    operation_pos: u32,
}

impl RouteDto {
    pub fn to_model(&self) -> ApplicationResult<Route> {
        Ok(Route::new(
            RouteId::from_string(self.id.clone()),
            &self.name,
            Polyline::from(self.waypoint_polyline.clone()).try_into()?,
            self.operation_pos as usize,
        ))
    }

    pub fn from_model(route: &Route) -> ApplicationResult<RouteDto> {
        Ok(RouteDto {
            id: route.id().to_string(),
            name: route.name().clone(),
            waypoint_polyline: Polyline::from(route.waypoints().clone()).into(),
            operation_pos: *route.op_num() as u32,
        })
    }
}
