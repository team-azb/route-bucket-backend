use serde::{Deserialize, Serialize};

use crate::model::user::UserId;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RouteSearchQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_id: Option<UserId>,
}

impl RouteSearchQuery {
    pub fn empty() -> Self {
        Default::default()
    }
}
