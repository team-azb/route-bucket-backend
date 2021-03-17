use bigdecimal::ToPrimitive;
use diesel::{
    associations::HasTable, BelongingToDsl, ExpressionMethods, MysqlConnection, QueryDsl,
    RunQueryDsl,
};
use std::convert::TryInto;

use crate::domain::coordinate::Coordinate;
use crate::domain::route::{Route, RouteRepository};
use crate::domain::types::RouteId;
use crate::infrastructure::dto::coordinate::CoordinateDto;
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::schema;
use crate::lib::error::{ApplicationError, ApplicationResult};

pub struct RouteRepositoryMysql {
    conn: MysqlConnection,
}

impl RouteRepositoryMysql {
    pub fn new(conn: MysqlConnection) -> RouteRepositoryMysql {
        RouteRepositoryMysql { conn }
    }

    pub fn route_to_dtos(route: &Route) -> (RouteDto, Vec<CoordinateDto>) {
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
        let route_dto = RouteDto::table()
            .filter(schema::routes::id.eq(&route_id.to_string()))
            .first::<RouteDto>(&self.conn)
            .or_else(|_| {
                Err(ApplicationError::ResourceNotFound {
                    resource_name: "Route",
                    id: route_id.to_string(),
                })
            })?;

        let coord_dtos = CoordinateDto::belonging_to(&route_dto)
            .load(&self.conn)
            .expect("Couldn't load coords");

        Ok(route_dto.to_model(coord_dtos)?)
    }

    fn register(&self, route: &Route) -> ApplicationResult<()> {
        let (route_dto, coord_dtos) = Self::route_to_dtos(route);

        diesel::insert_into(RouteDto::table())
            .values(route_dto)
            .execute(&self.conn)
            .expect("failed inserting route");

        diesel::insert_into(CoordinateDto::table())
            .values(coord_dtos)
            .execute(&self.conn)
            .expect("failed inserting coord");

        Ok(())
    }
}
