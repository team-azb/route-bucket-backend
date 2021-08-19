use std::convert::TryFrom;

use serde::Serialize;

use route_bucket_domain::model::{
    Coordinate, Distance, Elevation, Route, RouteGpx, RouteId, RouteInfo, Segment,
};
use route_bucket_utils::ApplicationError;

#[derive(Serialize)]
pub struct RouteGetResponse {
    #[serde(flatten)]
    pub route_info: RouteInfo,
    pub waypoints: Vec<Coordinate>,
    pub segments: Vec<Segment>,
    pub elevation_gain: Elevation,
    pub total_distance: Distance,
}

#[derive(Serialize)]
pub struct RouteGetAllResponse {
    #[serde(rename = "routes")]
    pub route_infos: Vec<RouteInfo>,
}

pub type RouteGetGpxResponse = RouteGpx;

#[derive(Serialize)]
pub struct RouteCreateResponse {
    pub id: RouteId,
}

#[derive(Serialize)]
pub struct RouteOperationResponse {
    pub waypoints: Vec<Coordinate>,
    pub segments: Vec<Segment>,
    pub elevation_gain: Elevation,
    pub total_distance: Distance,
}

impl TryFrom<Route> for RouteGetResponse {
    type Error = ApplicationError;

    fn try_from(route: Route) -> Result<Self, Self::Error> {
        let (info, _, seg_list) = route.into();
        Ok(RouteGetResponse {
            route_info: info,
            waypoints: seg_list.gather_waypoints(),
            elevation_gain: seg_list.calc_elevation_gain(),
            total_distance: seg_list.get_total_distance()?,
            segments: seg_list.clone().into_segments_in_between(),
        })
    }
}

impl TryFrom<Route> for RouteOperationResponse {
    type Error = ApplicationError;

    fn try_from(route: Route) -> Result<Self, Self::Error> {
        Ok(RouteOperationResponse {
            waypoints: route.gather_waypoints(),
            elevation_gain: route.calc_elevation_gain(),
            total_distance: route.get_total_distance()?,
            segments: route.into_segments_in_between(),
        })
    }
}
