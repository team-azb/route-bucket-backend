use derive_more::From;
use serde::Deserialize;
use validator::{Validate, ValidationError};

use route_bucket_domain::model::{
    permission::PermissionType,
    route::{Coordinate, DrawingMode},
    user::UserId,
};

#[derive(From, Deserialize, Validate)]
pub struct RouteCreateRequest {
    #[validate(length(min = 1, max = 50))]
    pub(super) name: String,
    pub(super) is_public: bool,
}

#[derive(From, Deserialize, Validate)]
pub struct NewPointRequest {
    pub(super) mode: DrawingMode,
    // TODO: Validate coordinate range.
    pub(super) coord: Coordinate,
}

#[derive(From, Deserialize, Validate)]
pub struct RemovePointRequest {
    pub(super) mode: DrawingMode,
}

#[derive(From, Deserialize, Validate)]
pub struct RouteRenameRequest {
    #[validate(length(min = 1, max = 50))]
    pub(super) name: String,
}

#[derive(From, Deserialize, Validate)]
pub struct UpdatePermissionRequest {
    #[validate]
    pub(super) user_id: UserId,
    #[serde(alias = "type")]
    #[validate(custom = "Self::validate_permission_type")]
    pub(super) permission_type: PermissionType,
}

impl UpdatePermissionRequest {
    // NOTE: validatorのrangeはnumber以外にはなぜか対応していない
    // 参考： https://github.com/Keats/validator/issues/204
    fn validate_permission_type(permission_type: &PermissionType) -> Result<(), ValidationError> {
        match *permission_type {
            PermissionType::Viewer | PermissionType::Editor => Ok(()),
            _ => Err(ValidationError::new("Only VIEWER and EDITOR are allowed.")),
        }
    }
}

#[derive(From, Deserialize, Validate)]
pub struct DeletePermissionRequest {
    #[validate]
    pub(super) user_id: UserId,
}
