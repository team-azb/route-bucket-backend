use derive_more::From;
use route_bucket_domain::model::user::UserId;
use serde::Serialize;

#[derive(Debug, Serialize, From)]
#[cfg_attr(test, derive(PartialEq))]
pub struct UserCreateResponse {
    pub id: UserId,
}
