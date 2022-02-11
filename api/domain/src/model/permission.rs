use derive_more::{From, Into};
use getset::Getters;
use serde::{Deserialize, Serialize};

use super::{route::RouteId, user::UserId};

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialOrd,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PermissionType {
    Viewer,
    Editor,
}

#[derive(Clone, Debug, From, Into, Getters)]
#[get = "pub"]
pub struct Permission {
    route_id: RouteId,
    user_id: UserId,
    permission_type: PermissionType,
}
