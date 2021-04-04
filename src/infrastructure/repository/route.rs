use diesel::{
    associations::HasTable,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    BelongingToDsl, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl,
};
use std::convert::TryInto;

use crate::domain::route::{Route, RouteRepository};
use crate::domain::types::RouteId;
use crate::infrastructure::dto::coordinate::CoordinateDto;
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::schema;
use crate::utils::error::{ApplicationError, ApplicationResult};

type MysqlConnectionManager = ConnectionManager<MysqlConnection>;

pub struct RouteRepositoryMysql {
    pool: Pool<MysqlConnectionManager>,
}

impl RouteRepositoryMysql {
    pub fn new(pool: Pool<MysqlConnectionManager>) -> RouteRepositoryMysql {
        RouteRepositoryMysql { pool }
    }

    fn get_connection(&self) -> ApplicationResult<PooledConnection<MysqlConnectionManager>> {
        let conn = self.pool.get().or_else(|_| {
            Err(ApplicationError::DataBaseError(
                "Failed to get DB connection.",
            ))
        })?;
        Ok(conn)
    }

    fn route_to_dtos(route: &Route) -> (RouteDto, Vec<CoordinateDto>) {
        let route_dto = RouteDto::from_model(route);
        let coord_dtos = route
            .points()
            .iter()
            .enumerate()
            .map(|(index, coord)| {
                CoordinateDto::from_model(coord, route.id(), index.try_into().unwrap())
            })
            .collect();
        (route_dto, coord_dtos)
    }
}

impl RouteRepository for RouteRepositoryMysql {
    fn find(&self, route_id: &RouteId) -> ApplicationResult<Route> {
        let conn = self.get_connection()?;
        let route_dto = RouteDto::table()
            .find(&route_id.to_string())
            .first::<RouteDto>(&conn)
            .or_else(|_| {
                Err(ApplicationError::ResourceNotFound {
                    resource_name: "Route",
                    id: route_id.to_string(),
                })
            })?;

        let coord_dtos = CoordinateDto::belonging_to(&route_dto)
            .order(schema::coordinates::index.asc())
            .load(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to load from Coordinates!",
                ))
            })?;

        Ok(route_dto.to_model(coord_dtos)?)
    }

    fn register(&self, route: &Route) -> ApplicationResult<()> {
        let conn = self.get_connection()?;

        let (route_dto, coord_dtos) = Self::route_to_dtos(route);

        diesel::insert_into(RouteDto::table())
            .values(route_dto)
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to insert into Routes!",
                ))
            })?;

        diesel::insert_into(CoordinateDto::table())
            .values(coord_dtos)
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to insert into Coordinates!",
                ))
            })?;

        Ok(())
    }
}
