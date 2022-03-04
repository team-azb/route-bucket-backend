use derive_more::{From, Into};
use getset::Getters;
use serde::{Deserialize, Serialize};

use super::{route::RouteId, user::UserId};

#[cfg(any(test, feature = "fixtures"))]
use derivative::Derivative;

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
#[cfg_attr(any(test, feature = "fixtures"), derive(Derivative))]
#[cfg_attr(any(test, feature = "fixtures"), derivative(PartialEq))]
pub struct Permission {
    #[cfg_attr(any(test, feature = "fixtures"), derivative(PartialEq = "ignore"))]
    route_id: RouteId,
    user_id: UserId,
    permission_type: PermissionType,
}

#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod tests {
    use crate::model::user::tests::UserIdFixtures;

    use super::*;

    pub trait PermissionFixtures {
        fn porzingis_viewer_permission() -> Permission {
            Permission {
                route_id: RouteId::new(),
                user_id: UserId::porzingis(),
                permission_type: PermissionType::Viewer,
            }
        }
    }

    impl PermissionFixtures for Permission {}
}
