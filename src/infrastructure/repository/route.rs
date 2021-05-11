use crate::domain::model::route::{Route, RouteRepository};
use crate::domain::model::types::RouteId;
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::repository::connection_pool::MysqlConnectionPool;
use crate::utils::error::{ApplicationError, ApplicationResult};
use diesel::{associations::HasTable, QueryDsl, RunQueryDsl};

pub struct RouteRepositoryMysql {
    pool: MysqlConnectionPool,
}

impl RouteRepositoryMysql {
    pub fn new() -> RouteRepositoryMysql {
        RouteRepositoryMysql {
            pool: MysqlConnectionPool::new(),
        }
    }
}

impl RouteRepository for RouteRepositoryMysql {
    fn find(&self, route_id: &RouteId) -> ApplicationResult<Route> {
        let conn = self.pool.get_connection()?;
        let route_dto = RouteDto::table()
            .find(&route_id.to_string())
            .first::<RouteDto>(&conn)
            .or_else(|_| {
                Err(ApplicationError::ResourceNotFound {
                    resource_name: "Route",
                    id: route_id.to_string(),
                })
            })?;

        Ok(route_dto.to_model()?)
    }

    fn find_all(&self) -> ApplicationResult<Vec<Route>> {
        let conn = self.pool.get_connection()?;

        let route_dtos = RouteDto::table().load::<RouteDto>(&conn).or_else(|_| {
            Err(ApplicationError::DataBaseError(
                "Failed to load from Routes!".into(),
            ))
        })?;

        Ok(route_dtos
            .iter()
            .map(|dto| dto.to_model())
            .collect::<ApplicationResult<Vec<Route>>>()?)
    }

    fn register(&self, route: &Route) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        let route_dto = RouteDto::from_model(route)?;

        diesel::insert_into(RouteDto::table())
            .values(route_dto)
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to insert into Routes!".into(),
                ))
            })?;

        Ok(())
    }

    fn update(&self, route: &Route) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        let route_dto = RouteDto::from_model(route)?;

        diesel::update(&route_dto)
            .set(&route_dto)
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(format!(
                    "Failed to update Route {}",
                    route.id()
                )))
            })?;

        Ok(())
    }

    fn delete(&self, route_id: &RouteId) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        let id_str = route_id.to_string();

        diesel::delete(RouteDto::table().find(&id_str))
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to delete Route!".into(),
                ))
            })?;

        Ok(())
    }
}
