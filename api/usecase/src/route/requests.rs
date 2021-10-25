use derive_more::From;
use serde::Deserialize;

use route_bucket_domain::model::route::{Coordinate, DrawingMode};

#[derive(From, Deserialize)]
pub struct RouteCreateRequest {
    pub(super) name: String,
}

#[derive(From, Deserialize)]
pub struct NewPointRequest {
    pub(super) mode: DrawingMode,
    pub(super) coord: Coordinate,
}

#[derive(From, Deserialize)]
pub struct RemovePointRequest {
    pub(super) mode: DrawingMode,
}

#[derive(From, Deserialize)]
pub struct RouteRenameRequest {
    pub(super) name: String,
}
