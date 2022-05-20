use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::model::user::UserId;

use super::RouteId;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate)]
#[cfg_attr(any(test, feature = "fixtures"), derive(PartialEq))]
pub struct RouteSearchQuery {
    #[serde(skip_deserializing)]
    pub ids: Option<Vec<RouteId>>,
    #[serde(skip_deserializing)]
    pub caller_id: Option<UserId>,
    #[validate]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<UserId>,
    #[serde(default)]
    pub page_offset: usize,
    pub page_size: Option<usize>,
    #[serde(default)]
    pub is_editable: bool,
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
        fn doncic_query(ids: Vec<RouteId>, is_editable: bool) -> RouteSearchQuery {
            RouteSearchQuery {
                ids: Some(ids),
                caller_id: Some(UserId::doncic()),
                ..Self::doncic_request(is_editable)
            }
        }

        fn doncic_request(is_editable: bool) -> RouteSearchQuery {
            RouteSearchQuery {
                owner_id: Some(UserId::doncic()),
                is_editable,
                ..Default::default()
            }
        }
    }

    impl RouteSearchQueryFixtures for RouteSearchQuery {}
}
