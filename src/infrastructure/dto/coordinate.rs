use bigdecimal::BigDecimal;

use crate::domain::polyline::Coordinate;
use crate::domain::types::RouteId;
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::schema::coordinates;
use crate::utils::error::ApplicationResult;

/// 座標のdto構造体
#[derive(Identifiable, Queryable, Insertable, Associations, Debug)]
#[table_name = "coordinates"]
#[primary_key(route_id, index)]
#[belongs_to(RouteDto, foreign_key = "route_id")]
pub struct CoordinateDto {
    route_id: String,
    index: u32,
    latitude: BigDecimal,
    longitude: BigDecimal,
}

impl CoordinateDto {
    pub fn to_model(&self) -> ApplicationResult<Coordinate> {
        Ok(Coordinate::new(
            self.latitude.clone(),
            self.longitude.clone(),
        )?)
    }

    pub fn from_model(coord: &Coordinate, route_id: &RouteId, index: u32) -> CoordinateDto {
        CoordinateDto {
            route_id: route_id.to_string(),
            index,
            latitude: coord.latitude().value(),
            longitude: coord.longitude().value(),
        }
    }
}
