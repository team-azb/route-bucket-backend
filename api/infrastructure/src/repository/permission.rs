use std::sync::Arc;

use async_trait::async_trait;
use route_bucket_domain::{
    model::{
        permission::{Permission, PermissionType},
        route::{RouteId, RouteInfo},
        user::UserId,
    },
    repository::{PermissionRepository, Repository},
};
use route_bucket_utils::ApplicationResult;
use sqlx::MySqlPool;
use tokio::sync::Mutex;

use crate::dto::permission::PermissionDto;

use super::{gen_err_mapper, RepositoryConnectionMySql};

pub struct PermissionRepositoryMySql(pub(super) Arc<MySqlPool>);

#[async_trait]
impl Repository for PermissionRepositoryMySql {
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
impl PermissionRepository for PermissionRepositoryMySql {
    async fn find_type(
        &self,
        route_info: &RouteInfo,
        user_id: &UserId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<PermissionType> {
        if *user_id == *route_info.owner_id() {
            return Ok(PermissionType::Owner);
        }

        let mut conn = conn.lock().await;

        let sqlx_result = sqlx::query_as::<_, PermissionDto>(
            r"
            SELECT * FROM permissions WHERE `route_id` = ? AND `user_id` = ?
            ",
        )
        .bind(route_info.id().to_string())
        .bind(user_id.to_string())
        .fetch_one(&mut *conn)
        .await;

        match sqlx_result {
            Ok(dto) => {
                let permission = dto.into_model()?;
                Ok(*permission.permission_type())
            }
            Err(sqlx::Error::RowNotFound) => Ok(PermissionType::None),
            Err(other_sqlx_err) => Err(gen_err_mapper("failed to find permission")(other_sqlx_err)),
        }
    }

    async fn insert_or_update(
        &self,
        permission: &Permission,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()> {
        let mut conn = conn.lock().await;
        let dto = PermissionDto::from_model(permission)?;

        sqlx::query(
            r"
            INSERT INTO permissions VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE `permission_type` = ?
            ",
        )
        .bind(dto.route_id())
        .bind(dto.user_id())
        .bind(dto.permission_type())
        .bind(dto.permission_type())
        .execute(&mut *conn)
        .await
        .map_err(gen_err_mapper("failed to insert or update Permission"))?;

        Ok(())
    }

    async fn delete(
        &self,
        route_id: &RouteId,
        user_id: &UserId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()> {
        let mut conn = conn.lock().await;

        sqlx::query(
            r"
            DELETE FROM permissions WHERE `route_id` = ? AND `user_id` = ?
            ",
        )
        .bind(route_id.to_string())
        .bind(user_id.to_string())
        .execute(&mut *conn)
        .await
        .map_err(gen_err_mapper("failed to delete Permission"))?;

        Ok(())
    }
}
