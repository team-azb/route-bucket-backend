use crate::domain::model::route::{Route, RouteInterpolationApi};
use crate::utils::error::ApplicationResult;
use std::collections::HashMap;

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
    fn interpolate(&self, route: &Route) -> ApplicationResult<String> {
        let target_url = format!(
            "{}/route/v1/bike/polyline({})?overview=full",
            self.api_root,
            route.polyline().encode()?
        );
        let resp = ureq::get(&target_url)
            .call()
            .unwrap()
            .into_json::<HashMap<String, HashMap<String, String>>>()
            .unwrap();
        println!("{}", resp["routes"]["geometry"]);
        todo!()
    }
}
