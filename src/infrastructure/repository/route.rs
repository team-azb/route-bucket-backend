use diesel::{MysqlConnection, QueryDsl, RunQueryDsl, BelongingToDsl, ExpressionMethods, associations::HasTable};

use crate::domain::route::{RouteRepository, Route};
use crate::domain::types::RouteId;
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::schema;
use crate::lib::error::{ApplicationError, ApplicationResult};
use crate::infrastructure::dto::coordinate::CoordinateDto;
use crate::domain::coordinate::Coordinate;
use bigdecimal::ToPrimitive;


pub struct RouteRepositoryMysql {
    conn: MysqlConnection
}

impl RouteRepositoryMysql {
    pub fn new(conn: MysqlConnection) -> RouteRepositoryMysql {
        RouteRepositoryMysql { conn }
    }

    fn dto_to_coord(coord_dto: &CoordinateDto) -> ApplicationResult<Coordinate> {
        Ok(Coordinate::create(
            coord_dto.latitude.to_f64().unwrap(),
            coord_dto.longitude.to_f64().unwrap())?)
    }

    fn dtos_to_route(route_dto: &RouteDto, coord_dtos: Vec<CoordinateDto>) -> ApplicationResult<Route> {
        Ok(
            Route::new(
                RouteId(route_dto.id.clone()),
                route_dto.name.clone(),
                coord_dtos.iter()
                    .map(|dto| Self::dto_to_coord(dto))
                    // Resultにcollectからのquestion
                    // https://users.rust-lang.org/t/use-operator-inside-map-function/16230/7
                    .collect::<ApplicationResult<Vec<_>>>()?
            )
        )
    }
}


impl RouteRepository for RouteRepositoryMysql {
    fn find(&self, route_id: RouteId) -> ApplicationResult<Route> {
        let route_dto = RouteDto::table()
            .filter(schema::routes::id.eq(&route_id.to_string()))
            .first::<RouteDto>(&self.conn)
            .or_else(|_| {
                Err(ApplicationError::ResourceNotFound {
                    resource_name: "Route",
                    id: route_id.to_string()
                })
            })?;

        let coord_dtos = CoordinateDto::belonging_to(&route_dto)
            .load(&self.conn)
            .expect("Couldn't load coords");

        Ok(Self::dtos_to_route(&route_dto, coord_dtos)?)
    }
}