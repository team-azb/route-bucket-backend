use async_trait::async_trait;
use sqlx::mysql::MySqlTransactionManager;
use sqlx::pool::PoolConnection;
use sqlx::{MySql, TransactionManager};
use tokio::sync::Mutex;

use crate::domain::repository::Connection;
use crate::utils::error::{ApplicationError, ApplicationResult};

pub mod route;

fn gen_err_mapper(msg: &'static str) -> impl FnOnce(sqlx::Error) -> ApplicationError {
    move |err| ApplicationError::DataBaseError(format!("{} ({:?})", msg, err))
}

#[async_trait]
impl Connection for Mutex<PoolConnection<MySql>> {
    async fn begin_transaction(&self) -> ApplicationResult<()> {
        let mut conn = self.lock().await;
        MySqlTransactionManager::begin(&mut *conn)
            .await
            .map_err(gen_err_mapper("failed to begin transaction"))
    }

    async fn commit_transaction(&self) -> ApplicationResult<()> {
        let mut conn = self.lock().await;
        MySqlTransactionManager::commit(&mut *conn)
            .await
            .map_err(gen_err_mapper("failed to commit transaction"))
    }

    async fn rollback_transaction(&self) -> ApplicationResult<()> {
        let mut conn = self.lock().await;
        MySqlTransactionManager::rollback(&mut *conn)
            .await
            .map_err(gen_err_mapper("failed to rollback transaction"))
    }
}
