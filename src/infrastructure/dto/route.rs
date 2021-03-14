use crate::infrastructure::schema::routes;

#[derive(Queryable, Insertable, Debug)]
#[table_name = "routes"]
pub struct RouteDto {
    pub id: String,
    pub name: String
}