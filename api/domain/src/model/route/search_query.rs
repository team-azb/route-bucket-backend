use serde::{Deserialize, Serialize};

use crate::model::user::UserId;

use super::RouteId;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fixtures"), derive(PartialEq))]
pub struct RouteSearchQuery {
    #[serde(skip_deserializing)]
    pub ids: Option<Vec<RouteId>>,
    #[serde(skip_deserializing)]
    pub caller_id: Option<UserId>,
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
        fn search_guest(ids: Option<Vec<RouteId>>, caller_id: Option<UserId>) -> RouteSearchQuery {
            RouteSearchQuery {
                ids,
                caller_id,
                owner_id: Some(UserId::doncic()),
                ..Default::default()
            }
        }
    }

    impl RouteSearchQueryFixtures for RouteSearchQuery {}
}
