use route_bucket_domain::model::user::UserId;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UserCreateResponse {
    pub id: UserId,
}

// TODO: mod testかく
