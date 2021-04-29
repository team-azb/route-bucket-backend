use diesel::{
    associations::HasTable,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    BelongingToDsl, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl,
};
use std::convert::TryInto;

use crate::domain::route::{Route, RouteRepository};
use crate::domain::types::RouteId;
use crate::infrastructure::dto::operation::OperationDto;
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

    pub fn get_connection(&self) -> ApplicationResult<PooledConnection<MysqlConnectionManager>> {
        let conn = self.pool.get().or_else(|_| {
            Err(ApplicationError::DataBaseError(
                "Failed to get DB connection.".into(),
            ))
        })?;
        Ok(conn)
    }

    fn route_to_dtos(route: &Route) -> ApplicationResult<(RouteDto, Vec<OperationDto>)> {
        let route_dto = RouteDto::from_model(route)?;
        let op_dtos = route
            .operation_history()
            .op_list()
            .iter()
            .enumerate()
            .map(|(index, op)| OperationDto::from_model(op, route.id(), index.try_into().unwrap()))
            .collect::<ApplicationResult<Vec<_>>>()?;
        Ok((route_dto, op_dtos))
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

        // TODO: OperationRepositoryとして分離する
        let op_dtos = OperationDto::belonging_to(&route_dto)
            .order(schema::operations::index.asc())
            .load(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to load from Operations!".into(),
                ))
            })?;

        Ok(route_dto.to_model(op_dtos)?)
    }

    fn find_all(&self) -> ApplicationResult<Vec<Route>> {
        let conn = self.get_connection()?;

        let route_dtos = RouteDto::table().load::<RouteDto>(&conn).or_else(|_| {
            Err(ApplicationError::DataBaseError(
                "Failed to load from Routes!".into(),
            ))
        })?;

        // TODO: Routeじゃなく、RouteProfile的なOperationHistoryを抜いたstructを返す
        //     : ここではOperationを空のままRouteにしてしまっている
        Ok(route_dtos
            .iter()
            .map(|dto| dto.to_model(Vec::new()))
            .collect::<ApplicationResult<Vec<Route>>>()?)
    }

    fn register(&self, route: &Route) -> ApplicationResult<()> {
        let conn = self.get_connection()?;

        let (route_dto, op_dtos) = Self::route_to_dtos(route)?;

        diesel::insert_into(RouteDto::table())
            .values(route_dto)
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to insert into Routes!".into(),
                ))
            })?;

        // TODO: OperationRepositoryとして分離する
        diesel::insert_into(OperationDto::table())
            .values(op_dtos)
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to insert into Operations!".into(),
                ))
            })?;

        Ok(())
    }

    fn update(&self, route: &Route) -> ApplicationResult<()> {
        let conn = self.get_connection()?;

        let (route_dto, op_dtos) = Self::route_to_dtos(route)?;

        diesel::update(&route_dto)
            .set(&route_dto)
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(format!(
                    "Failed to update Route {}",
                    route.id()
                )))
            })?;

        // TODO: 現状対応する操作を全削除してinsertし直すという間抜けな方法をとっている
        //     : これはMySQLのupsertがdieselでできないため(postgresのやつは使えるっぽい）
        diesel::delete(OperationDto::belonging_to(&route_dto))
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(format!(
                    "Failed to delete Operations that belong to Route {}",
                    route.id()
                )))
            })?;
        diesel::insert_into(OperationDto::table())
            .values(op_dtos)
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to insert into Operations!".into(),
                ))
            })?;

        Ok(())
    }

    fn delete(&self, route_id: &RouteId) -> ApplicationResult<()> {
        let conn = self.get_connection()?;

        let id_str = route_id.to_string();

        diesel::delete(RouteDto::table().find(&id_str))
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to delete Route!".into(),
                ))
            })?;

        // TODO: OperationRepositoryとして分離する
        diesel::delete(OperationDto::table().filter(schema::operations::route_id.eq(&id_str)))
            .execute(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to delete Operations!".into(),
                ))
            })?;

        Ok(())
    }
}
