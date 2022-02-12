use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Deref;
use route_bucket_domain::repository::Connection;
use route_bucket_utils::{ApplicationError, ApplicationResult};
use sqlx::mysql::{MySqlPoolOptions, MySqlTransactionManager};
use sqlx::pool::PoolConnection;
use sqlx::{MySql, TransactionManager};
use tokio::sync::Mutex;

use self::route::RouteRepositoryMySql;
use self::user::UserRepositoryMySql;

pub mod route;
pub mod user;

pub async fn init_repositories() -> (RouteRepositoryMySql, UserRepositoryMySql) {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");
    let pool = Arc::new(
        MySqlPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
            .unwrap(),
    );
    (
        RouteRepositoryMySql(pool.clone()),
        UserRepositoryMySql(pool),
    )
}

fn gen_err_mapper(msg: &'static str) -> impl FnOnce(sqlx::Error) -> ApplicationError {
    move |err| ApplicationError::DataBaseError(format!("{} ({:?})", msg, err))
}

#[derive(Deref)]
pub struct RepositoryConnectionMySql(Mutex<PoolConnection<MySql>>);

#[async_trait]
impl Connection for RepositoryConnectionMySql {
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
