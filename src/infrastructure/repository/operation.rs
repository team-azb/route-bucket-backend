use crate::domain::model::operation::{Operation, OperationRepository};
use crate::domain::model::types::RouteId;
use crate::infrastructure::dto::operation::OperationDto;
use crate::infrastructure::repository::connection_pool::MysqlConnectionPool;
use crate::infrastructure::schema;
use crate::utils::error::{ApplicationError, ApplicationResult};
use diesel::{associations::HasTable, ExpressionMethods, QueryDsl, RunQueryDsl};

pub struct OperationRepositoryMysql {
    pool: MysqlConnectionPool,
}

impl OperationRepositoryMysql {
    pub fn new() -> Self {
        Self {
            pool: MysqlConnectionPool::new(),
        }
    }

    fn models_to_dtos(
        op_list: &Vec<Operation>,
        route_id: &RouteId,
    ) -> ApplicationResult<Vec<OperationDto>> {
        op_list
            .iter()
            .enumerate()
            .map(|(i, op)| OperationDto::from_model(op, route_id, i as u32))
            .collect::<ApplicationResult<Vec<_>>>()
    }

    fn dtos_to_models(op_dtos: &Vec<OperationDto>) -> ApplicationResult<Vec<Operation>> {
        op_dtos
            .iter()
            .map(OperationDto::to_model)
            .collect::<ApplicationResult<Vec<_>>>()
    }
}

impl OperationRepository for OperationRepositoryMysql {
    fn find_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<Vec<Operation>> {
        let conn = self.pool.get_connection()?;

        let op_dtos = OperationDto::table()
            .filter(schema::operations::route_id.eq(&route_id.to_string()))
            .order(schema::operations::index.asc())
            .load(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to load from Operations!".into(),
                ))
            })?;

        Self::dtos_to_models(&op_dtos)
    }

    fn update_by_route_id(
        &self,
        route_id: &RouteId,
        op_list: &Vec<Operation>,
    ) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        // TODO: 現状対応する操作を全削除してinsertし直すという間抜けな方法をとっている
        //     : これはMySQLのupsertがdieselでできないため(postgresのやつは使えるっぽい）
        self.delete_by_route_id(route_id)?;

        let op_dtos = Self::models_to_dtos(op_list, route_id)?;

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

    fn delete_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;
        let source =
            OperationDto::table().filter(schema::operations::route_id.eq(route_id.to_string()));

        diesel::delete(source).execute(&conn).or_else(|_| {
            Err(ApplicationError::DataBaseError(format!(
                "Failed to delete Operations that belong to Route {}",
                route_id.to_string()
            )))
        })?;

        Ok(())
    }
}
