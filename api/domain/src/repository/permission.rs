use async_trait::async_trait;
use route_bucket_utils::ApplicationResult;

use crate::model::{
    permission::{Permission, PermissionType},
    route::{RouteId, RouteInfo},
    user::UserId,
};

use super::Repository;

#[async_trait]
pub trait PermissionRepository: Repository {
    async fn authorize_user(
        &self,
        route_info: &RouteInfo,
        user_id: &UserId,
        target_type: PermissionType,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;

    async fn insert_or_update(
        &self,
        permission: &Permission,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;

    async fn delete(
        &self,
        route_id: &RouteId,
        user_id: &UserId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;
}

pub trait CallPermissionRepository {
    type PermissionRepository: PermissionRepository;

    fn permission_repository(&self) -> &Self::PermissionRepository;
}

#[cfg(feature = "mocking")]
mockall::mock! {
    pub PermissionRepository {}

    #[async_trait]
    impl Repository for PermissionRepository {
        type Connection = super::MockConnection;

        async fn get_connection(&self) -> ApplicationResult<super::MockConnection>;
    }

    #[async_trait]
    impl PermissionRepository for PermissionRepository {
        async fn authorize_user(&self, route_info: &RouteInfo, user_id: &UserId, target_type: PermissionType, conn: &super::MockConnection) -> ApplicationResult<()>;

        async fn insert_or_update(&self, permission: &Permission, conn: &super::MockConnection) -> ApplicationResult<()>;

        async fn delete(&self, route_id: &RouteId, user_id: &UserId, conn: &super::MockConnection) -> ApplicationResult<()>;
    }
}
