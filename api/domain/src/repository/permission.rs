use async_trait::async_trait;
use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::{
    permission::{Permission, PermissionType},
    route::{RouteId, RouteInfo},
    user::UserId,
};

use super::Repository;

#[async_trait]
pub trait PermissionRepository: Repository {
    async fn find_type(
        &self,
        route_info: &RouteInfo,
        user_id: Option<UserId>,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<PermissionType>;

    async fn find_by_user_id(
        &self,
        user_id: &UserId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<Vec<Permission>>;

    async fn authorize_user(
        &self,
        route_info: &RouteInfo,
        user_id: Option<UserId>,
        target_type: PermissionType,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()> {
        let permission_type = self.find_type(route_info, user_id.clone(), conn).await?;

        (target_type <= permission_type).then(|| ()).ok_or_else(|| {
            let subject = if let Some(id) = user_id {
                format!("User {}", id)
            } else {
                "Unauthenticated user".into()
            };
            ApplicationError::AuthorizationError(format!(
                "{} doesn't have {} permission on Route {} (actual permission: {}).",
                subject,
                target_type,
                route_info.id(),
                permission_type
            ))
        })
    }

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
        async fn find_type(&self, route_info: &RouteInfo, user_id: Option<UserId>, conn: &super::MockConnection) -> ApplicationResult<PermissionType>;

        async fn find_by_user_id(&self, user_id: &UserId, conn: &super::MockConnection) -> ApplicationResult<Vec<Permission>>;

        async fn authorize_user(&self, route_info: &RouteInfo, user_id: Option<UserId>, target_type: PermissionType, conn: &super::MockConnection) -> ApplicationResult<()>;

        async fn insert_or_update(&self, permission: &Permission, conn: &super::MockConnection) -> ApplicationResult<()>;

        async fn delete(&self, route_id: &RouteId, user_id: &UserId, conn: &super::MockConnection) -> ApplicationResult<()>;
    }
}
