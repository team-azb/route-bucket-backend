use derive_more::From;
use serde::Deserialize;

use route_bucket_domain::model::{
    permission::PermissionType,
    route::{Coordinate, DrawingMode},
    user::UserId,
};

#[derive(From, Deserialize)]
pub struct RouteCreateRequest {
    pub(super) name: String,
}

#[derive(From, Deserialize)]
pub struct NewPointRequest {
    pub(super) mode: DrawingMode,
    pub(super) coord: Coordinate,
}

#[derive(From, Deserialize)]
pub struct RemovePointRequest {
    pub(super) mode: DrawingMode,
}

#[derive(From, Deserialize)]
pub struct RouteRenameRequest {
    pub(super) name: String,
}

#[derive(From, Deserialize)]
pub struct UpdatePermissionRequest {
    pub(super) user_id: UserId,
    #[serde(alias = "type")]
    pub(super) permission_type: PermissionType,
}

#[derive(From, Deserialize)]
pub struct DeletePermissionRequest {
    pub(super) user_id: UserId,
}
