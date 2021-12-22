use serde::{Deserialize, Serialize};

use crate::model::user::UserId;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fixtures"), derive(PartialEq))]
pub struct RouteSearchQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<UserId>,
    #[serde(default)]
    pub page_offset: usize,
    pub page_size: Option<usize>,
}

impl RouteSearchQuery {
    pub fn empty() -> Self {
        Default::default()
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod tests {
    use crate::model::user::tests::UserIdFixtures;

    use super::*;

    pub trait RouteSearchQueryFixtures {
        fn search_guest() -> RouteSearchQuery {
            RouteSearchQuery {
                owner_id: Some(UserId::doncic()),
                ..Default::default()
            }
        }
    }

    impl RouteSearchQueryFixtures for RouteSearchQuery {}
}
