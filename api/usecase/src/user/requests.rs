use std::convert::TryFrom;

use chrono::{NaiveDate, Utc};
use derive_more::From;
use route_bucket_utils::ApplicationError;
use serde::Deserialize;
use validator::Validate;

use route_bucket_domain::model::{
    types::{Email, Url},
    user::{Gender, User, UserId},
};

#[derive(From, Deserialize, Validate)]
pub struct UserCreateRequest {
    #[validate]
    id: UserId,
    #[validate(length(min = 1, max = 50))]
    name: String,
    #[validate]
    email: Email,
    #[serde(default)]
    gender: Gender,
    #[validate(custom = "UserCreateRequest::validate_birthdate")]
    birthdate: Option<NaiveDate>,
    #[validate]
    icon_url: Option<Url>,
    #[validate(length(min = 6))]
    password: String,
    #[validate(must_match = "password")]
    password_confirmation: String,
}

impl UserCreateRequest {
    fn validate_birthdate(birthdate: &NaiveDate) -> Result<(), validator::ValidationError> {
        if *birthdate <= Utc::today().naive_utc() {
            Ok(())
        } else {
            Err(validator::ValidationError::new("FUTURE_BIRTHDATE"))
        }
    }
}

impl TryFrom<UserCreateRequest> for (User, Email, String) {
    type Error = ApplicationError;

    fn try_from(value: UserCreateRequest) -> Result<Self, Self::Error> {
        value.validate()?;
        Ok((
            User::new(
                value.id,
                value.name,
                value.gender,
                value.birthdate,
                value.icon_url,
            ),
            value.email,
            value.password,
        ))
    }
}
