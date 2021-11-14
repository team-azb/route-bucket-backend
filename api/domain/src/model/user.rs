use chrono::NaiveDate;
use derive_more::{Constructor, Display, From, Into};
use getset::Getters;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::types::Url;

static USER_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9]?|[\-]?([a-zA-Z0-9])){0,38}$").unwrap());

#[derive(Clone, Debug, Serialize, Deserialize, Display, From, Into, Validate)]
#[display(fmt = "{}", id)]
#[serde(transparent)]
pub struct UserId {
    #[validate(regex = "USER_ID_REGEX")]
    id: String,
}

#[derive(Clone, Debug)]
pub struct UserAuthInfo {
    id: UserId,
    token: String,
}

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, strum::Display, strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Others,
}

impl Default for Gender {
    fn default() -> Self {
        Gender::Others
    }
}

#[derive(Clone, Debug, Constructor, Into, Getters)]
#[get = "pub"]
pub struct User {
    id: UserId,
    name: String,
    gender: Gender,
    birthdate: Option<NaiveDate>,
    icon_url: Option<Url>,
}
