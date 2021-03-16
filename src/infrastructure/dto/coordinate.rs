use crate::infrastructure::schema::coordinates;
use crate::infrastructure::dto::route::RouteDto;
use bigdecimal::BigDecimal;


/// 座標のdto構造体
#[derive(Identifiable, Queryable, Insertable, Associations, Debug)]
#[table_name = "coordinates"]
#[primary_key(route_id, index)]
#[belongs_to(RouteDto, foreign_key = "route_id")]
pub struct CoordinateDto {
    pub route_id: String,
    pub index: u32,
    pub latitude: BigDecimal,
    pub longitude: BigDecimal,
}