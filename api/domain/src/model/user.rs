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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Into, Validate)]
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

#[derive(Clone, Debug, Serialize, Deserialize, Constructor, Into, Getters, Setters)]
#[get = "pub"]
#[set = "pub"]
#[cfg_attr(any(test, feature = "fixtures"), derive(PartialEq))]
pub struct User {
    id: UserId,
    name: String,
    gender: Gender,
    birthdate: Option<NaiveDate>,
    icon_url: Option<Url>,
}

#[cfg(any(test, feature = "fixtures"))]
pub(crate) mod tests {
    use std::{convert::TryFrom, str::FromStr};

    use super::*;

    pub trait UserIdFixtures {
        fn doncic() -> UserId {
            UserId::from("luka7doncic".to_string())
        }

        fn porzingis() -> UserId {
            UserId::from("kporzee".to_string())
        }
    }

    impl UserIdFixtures for UserId {}

    pub trait UserFixtures {
        fn doncic() -> User {
            User {
                id: UserId::doncic(),
                name: "Luka Doncic".to_string(),
                gender: Gender::Male,
                birthdate: NaiveDate::from_str("1999-02-28").ok(),
                icon_url: Url::try_from("https://on.nba.com/30qMUEI".to_string()).ok(),
            }
        }

        fn porzingis() -> User {
            User {
                id: UserId::porzingis(),
                name: "Kristaps Porzingis".to_string(),
                // He's a male but I guess he didn't set his profile...
                gender: Gender::Others,
                birthdate: None,
                icon_url: None,
            }
        }
    }

    impl UserFixtures for User {}
}
