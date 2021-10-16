use std::convert::TryFrom;

use serde::Serialize;

use route_bucket_domain::model::{
    BoundingBox, Coordinate, Distance, Elevation, Route, RouteGpx, RouteId, RouteInfo, Segment,
};
use route_bucket_utils::ApplicationError;

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct RouteGetResponse {
    #[serde(flatten)]
    pub route_info: RouteInfo,
    pub waypoints: Vec<Coordinate>,
    pub segments: Vec<Segment>,
    pub elevation_gain: Elevation,
    pub total_distance: Distance,
    pub bounding_box: Option<BoundingBox>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct RouteGetAllResponse {
    #[serde(rename = "routes")]
    pub route_infos: Vec<RouteInfo>,
}

pub type RouteGetGpxResponse = RouteGpx;

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct RouteCreateResponse {
    pub id: RouteId,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(PartialEq))]
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
            bounding_box: (!seg_list.is_empty())
                .then(|| seg_list.calc_bounding_box())
                .transpose()?,
            segments: seg_list.into_segments_in_between(),
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

#[cfg(test)]
mod tests {
    use route_bucket_domain::model::fixtures::{
        BoundingBoxFixture, CoordinateFixtures, RouteFixtures, RouteInfoFixtures, SegmentFixtures,
    };
    use route_bucket_domain::model::DrawingMode;
    use rstest::rstest;
    use std::convert::TryInto;

    use super::*;

    fn empty_route_get_resp() -> RouteGetResponse {
        RouteGetResponse {
            route_info: RouteInfo::route0(0),
            waypoints: Vec::new(),
            elevation_gain: Elevation::zero(),
            total_distance: Distance::zero(),
            segments: Vec::new(),
            bounding_box: None,
        }
    }

    fn full_route_get_resp() -> RouteGetResponse {
        let dist = 26936.42633640023;
        RouteGetResponse {
            route_info: RouteInfo::route0(3),
            waypoints: Coordinate::yokohama_to_chiba_via_tokyo_coords(false, None),
            elevation_gain: 10.try_into().unwrap(),
            total_distance: 58759.973932514884.try_into().unwrap(),
            segments: vec![
                Segment::yokohama_to_tokyo(true, Some(0.), false, DrawingMode::Freehand),
                Segment::tokyo_to_chiba(true, Some(dist), false, DrawingMode::Freehand),
            ],
            bounding_box: Some(BoundingBox::yokohama_to_chiba_via_tokyo()),
        }
    }

    fn empty_route_operation_resp() -> RouteOperationResponse {
        RouteOperationResponse {
            waypoints: Vec::new(),
            elevation_gain: Elevation::zero(),
            total_distance: Distance::zero(),
            segments: Vec::new(),
        }
    }

    fn full_route_operation_resp() -> RouteOperationResponse {
        let dist = 26936.42633640023;
        RouteOperationResponse {
            waypoints: Coordinate::yokohama_to_chiba_via_tokyo_coords(false, None),
            elevation_gain: 10.try_into().unwrap(),
            total_distance: 58759.973932514884.try_into().unwrap(),
            segments: vec![
                Segment::yokohama_to_tokyo(true, Some(0.), false, DrawingMode::Freehand),
                Segment::tokyo_to_chiba(true, Some(dist), false, DrawingMode::Freehand),
            ],
        }
    }

    #[rstest]
    #[case::empty(Route::empty(), empty_route_get_resp())]
    #[case::full(
        Route::yokohama_to_chiba_via_tokyo_filled(true, true),
        full_route_get_resp()
    )]
    fn can_convert_route_to_get_resp(
        #[case] route: Route,
        #[case] expected_resp: RouteGetResponse,
    ) {
        assert_eq!(route.try_into(), Ok(expected_resp));
    }

    #[rstest]
    #[case::empty(Route::empty(), empty_route_operation_resp())]
    #[case::full(
        Route::yokohama_to_chiba_via_tokyo_filled(true, true),
        full_route_operation_resp()
    )]
    fn can_convert_route_to_operation_resp(
        #[case] route: Route,
        #[case] expected_resp: RouteOperationResponse,
    ) {
        assert_eq!(route.try_into(), Ok(expected_resp));
    }
}
