use crate::infrastructure::schema::routes;

/// ルートのdto構造体
#[derive(Identifiable, Queryable, Insertable, Debug)]
#[table_name = "routes"]
pub struct RouteDto {
    pub id: String,
    pub name: String
}