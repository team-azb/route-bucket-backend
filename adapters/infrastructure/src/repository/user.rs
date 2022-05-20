use std::sync::Arc;

use async_trait::async_trait;
use route_bucket_domain::{
    model::user::{User, UserId},
    repository::{Repository, UserRepository},
};
use route_bucket_utils::ApplicationResult;
use sqlx::MySqlPool;
use tokio::sync::Mutex;

use crate::dto::user::UserDto;

use super::{gen_err_mapper, RepositoryConnectionMySql};

pub struct UserRepositoryMySql(pub(super) Arc<MySqlPool>);

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
    async fn find(
        &self,
        id: &UserId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<User> {
        let mut conn = conn.lock().await;

        sqlx::query_as::<_, UserDto>(
            r"
            SELECT * FROM users WHERE id = ? FOR UPDATE
            ",
        )
        .bind(id.to_string())
        .fetch_one(&mut *conn)
        .await
        .map_err(gen_err_mapper("failed to find user"))?
        .into_model()
    }

    async fn insert(
        &self,
        user: &User,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()> {
        let mut conn = conn.lock().await;
        let dto = UserDto::from_model(user)?;

        sqlx::query(
            r"
            INSERT INTO users VALUES (?, ?, ?, ?, ?)
            ",
        )
        .bind(dto.id)
        .bind(dto.name)
        .bind(dto.gender)
        .bind(dto.birthdate)
        .bind(dto.icon_url)
        .execute(&mut *conn)
        .await
        .map_err(gen_err_mapper("failed to insert User"))?;
        Ok(())
    }

    async fn update(&self, user: &User, conn: &Self::Connection) -> ApplicationResult<()> {
        let mut conn = conn.lock().await;
        let dto = UserDto::from_model(user)?;

        sqlx::query(
            r"
            UPDATE users
            SET name = ?, gender = ?, birthdate = ?, icon_url = ?
            WHERE id = ?
            ",
        )
        .bind(dto.name)
        .bind(dto.gender)
        .bind(dto.birthdate)
        .bind(dto.icon_url)
        .bind(dto.id)
        .execute(&mut *conn)
        .await
        .map_err(gen_err_mapper("failed to update User"))?;

        Ok(())
    }

    async fn delete(
        &self,
        id: &UserId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()> {
        let mut conn = conn.lock().await;

        sqlx::query(
            r"
            DELETE FROM users WHERE id = ?
            ",
        )
        .bind(id.to_string())
        .execute(&mut *conn)
        .await
        .map_err(gen_err_mapper("failed to delete user"))?;

        Ok(())
    }
}
