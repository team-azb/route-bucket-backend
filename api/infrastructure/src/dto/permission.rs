use std::str::FromStr;

use getset::Getters;
use route_bucket_domain::model::{
    permission::{Permission, PermissionType},
    route::RouteId,
    user::UserId,
};
use route_bucket_utils::{ApplicationError, ApplicationResult};

/// ルートのdto構造体
#[derive(sqlx::FromRow, Getters)]
#[get = "pub"]
pub(crate) struct PermissionDto {
    pub(crate) route_id: String,
    pub(crate) user_id: String,
    pub(crate) permission_type: String,
}

impl PermissionDto {
    pub fn into_model(self) -> ApplicationResult<Permission> {
        let Self {
            user_id,
            route_id,
            permission_type,
        } = self;
        Ok(Permission::from((
            RouteId::from_string(route_id),
            UserId::from(user_id),
            PermissionType::from_str(&permission_type).map_err(|e| {
                ApplicationError::DataBaseError(format!(
                    "Failed to parse permission.permission_type ({:?})",
                    e
                ))
            })?,
        )))
    }

    pub fn from_model(permission: &Permission) -> ApplicationResult<Self> {
        let (route_id, user_id, permission_type) = permission.clone().into();
        Ok(Self {
            route_id: route_id.to_string(),
            user_id: user_id.to_string(),
            permission_type: permission_type.to_string(),
        })
    }
}
