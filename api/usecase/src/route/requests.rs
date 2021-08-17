use derive_more::From;
use serde::Deserialize;

use route_bucket_domain::model::Coordinate;

#[derive(From, Deserialize)]
pub struct RouteCreateRequest {
    pub(super) name: String,
}

#[derive(From, Deserialize)]
pub struct NewPointRequest {
    pub(super) coord: Coordinate,
}

#[derive(From, Deserialize)]
pub struct RouteRenameRequest {
    pub(super) name: String,
}
