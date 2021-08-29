use async_trait::async_trait;
use futures::FutureExt;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use tokio::sync::Mutex;

use route_bucket_domain::model::{User, UserId};
use route_bucket_domain::repository::{Repository, UserRepository};
use route_bucket_utils::ApplicationResult;

use crate::dto::user::UserDto;
use crate::repository::{gen_err_mapper, RepositoryConnectionMySql};

// TODO: Arcで囲んで、Routeの方と共有し、newも共通化する
pub struct UserRepositoryMySql(MySqlPool);

impl UserRepositoryMySql {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");
        MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .map(|res| res.map(Self))
            .await
            .unwrap()
    }
}

#[async_trait]
impl Repository for UserRepositoryMySql {
    type Connection = RepositoryConnectionMySql;

    async fn get_connection(&self) -> ApplicationResult<Self::Connection> {
        self.0
            .acquire()
            .await
            .map(Mutex::new)
            .map(RepositoryConnectionMySql)
            .map_err(gen_err_mapper("failed to get connection"))
    }
}

#[async_trait]
impl UserRepository for UserRepositoryMySql {
    async fn find(&self, id: &UserId, conn: &Self::Connection) -> ApplicationResult<User> {
        let mut conn = conn.lock().await;

        Ok(sqlx::query_as::<_, UserDto>(
            r"
            SELECT * FROM users WHERE id = ?
            ",
        )
        .bind(id.to_string())
        .fetch_one(&mut *conn)
        .await
        .map_err(gen_err_mapper("failed to find user"))?
        .into())
    }

    async fn insert(&self, user: &User, conn: &Self::Connection) -> ApplicationResult<()> {
        let mut conn = conn.lock().await;
        let dto = UserDto::from(user.clone());

        sqlx::query(
            r"
            INSERT INTO users VALUES (?)
            ",
        )
        .bind(dto.id)
        .execute(&mut *conn)
        .await
        .map_err(gen_err_mapper("failed to insert User"))?;
        Ok(())
    }
}
