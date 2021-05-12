use crate::domain::model::route::{Route, RouteInterpolationApi};
use crate::domain::model::types::Polyline;
use crate::utils::error::ApplicationResult;

/// osrmでルート補間をするための構造体
pub struct OsrmApi {
    api_root: String,
}

impl OsrmApi {
    pub fn new() -> Self {
        Self {
            api_root: std::env::var("OSRM_ROOT").expect("OSRM_ROOT NOT FOUND"),
        }
    }
}

impl RouteInterpolationApi for OsrmApi {
    fn interpolate(&self, route: &Route) -> ApplicationResult<Polyline> {
        let target_url = format!(
            "{}/route/v1/bike/polyline({})?overview=full",
            self.api_root,
            String::from(Polyline::from(route.waypoints().clone()))
        );
        let result = reqwest::blocking::get(&target_url)
            .map(|resp| resp.json::<serde_json::Value>().unwrap())
            .map_or(Polyline::from(route.waypoints().clone()), |map| {
                Polyline::from(
                    serde_json::from_value::<String>(map["routes"][0]["geometry"].clone()).unwrap(),
                )
            });

        Ok(result)
    }
}
