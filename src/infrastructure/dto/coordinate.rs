#[derive(Queryable)]
pub struct CoordinateDto {
    pub route_id: String,
    pub index: i32,
    pub latitude: f64,
    pub longitude: f64,
}