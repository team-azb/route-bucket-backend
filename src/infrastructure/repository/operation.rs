use crate::domain::operation::{Operation, OperationHistory, OperationRepository};
use crate::domain::types::RouteId;
use crate::infrastructure::dto::operation::OperationDto;
use crate::infrastructure::repository::connection_pool::MysqlConnectionPool;
use crate::infrastructure::schema;
use crate::utils::error::{ApplicationError, ApplicationResult};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;

pub struct OperationRepositoryMysql {
    pool: MysqlConnectionPool,
}

impl OperationRepositoryMysql {
    pub fn new() -> Self {
        Self {
            pool: MysqlConnectionPool::new(),
        }
    }

    fn dtos_to_models(dtos: &Vec<OperationDto>) -> ApplicationResult<Vec<Operation>> {
        dtos.iter()
            .map(OperationDto::to_model)
            .collect::<ApplicationResult<Vec<_>>>()
    }
}

impl OperationRepository for OperationRepositoryMysql {
    fn find_history(&self, route_id: &RouteId) -> ApplicationResult<Vec<Operation>> {
        let conn = self.pool.get_connection()?;

        let op_dtos = OperationDto::belonging_to(&route_dto)
            .order(schema::operations::index.asc())
            .load(&conn)
            .or_else(|_| {
                Err(ApplicationError::DataBaseError(
                    "Failed to load from Operations!".into(),
                ))
            })?;

        Self::dtos_to_history(op_dtos)
    }

    fn update_history(
        &self,
        route_id: &RouteId,
        history: &OperationHistory,
    ) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        // TODO: 現状対応する操作を全削除してinsertし直すという間抜けな方法をとっている
        //     : これはMySQLのupsertがdieselでできないため(postgresのやつは使えるっぽい）
        self.delete_history(route_id)?;

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

    fn delete_history(&self, route_id: &RouteId) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;
        let source =
            OperationDto::table().filter(schema::operations::route_id.eq(&route_id.to_string()));

        diesel::delete(source).execute(&conn).or_else(|_| {
            Err(ApplicationError::DataBaseError(format!(
                "Failed to delete Operations that belong to Route {}",
                route_id.to_string()
            )))
        })?;

        Ok(())
    }
}
