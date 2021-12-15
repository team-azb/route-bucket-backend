use std::collections::HashMap;

use derive_more::From;
use route_bucket_domain::model::user::UserId;
use serde::Serialize;

#[derive(Debug, Serialize, From)]
#[cfg_attr(test, derive(PartialEq))]
pub struct UserCreateResponse {
    pub id: UserId,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) enum ValidationErrorCode {
    InvalidFormat,
    AlreadyExists,
}

#[derive(Debug, Default, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct UserValidateResponse(pub(super) HashMap<&'static str, ValidationErrorCode>);
