use derive_more::{From, Into};
use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::model::types::UserId;

#[derive(Clone, Debug, Serialize, Deserialize, From, Into, Getters)]
#[get = "pub"]
pub struct User {
    id: UserId,
}
