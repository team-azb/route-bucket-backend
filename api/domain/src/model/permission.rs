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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "snake_case")]
pub enum PermissionType {
    None,
    Viewer,
    Editor,
    Owner,
}

#[derive(Clone, Debug, From, Into, Getters)]
#[get = "pub"]
#[cfg_attr(any(test, feature = "fixtures"), derive(PartialEq))]
pub struct Permission {
    route_id: RouteId,
    user_id: UserId,
    permission_type: PermissionType,
}
