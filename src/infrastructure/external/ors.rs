use crate::domain::model::route::{Route, RouteInterpolationApi};
use crate::domain::model::types::Polyline;
use crate::utils::error::ApplicationResult;

/// osrmでルート補間をするための構造体
pub struct OrsApi {
    api_root: String,
}

impl OrsApi {
    pub fn new() -> Self {
        Self {
            api_root: std::env::var("ORS_ROOT").expect("ORS_ROOT NOT FOUND"),
        }
    }
}

impl RouteInterpolationApi for OrsApi {
    fn interpolate(&self, route: &Route) -> ApplicationResult<Polyline> {
        let target_url = format!(
            "{}/route/v1/bike/polyline({})?overview=full",
            self.api_root,
            String::from(Polyline::from(route.waypoints().clone()))
        );
        let result = reqwest::blocking::get(target_url);

        // まだif let から&&で繋げない(https://github.com/rust-lang/rust/issues/53667)
        let polyline = if let Ok(resp) = result {
            if resp.status().is_success() {
                let json = resp.json::<serde_json::Value>().unwrap();
                Polyline::from(
                    serde_json::from_value::<String>(json["routes"][0]["geometry"].clone())
                        .unwrap(),
                )
            } else {
                Polyline::from(route.waypoints().clone())
            }
        } else {
            Polyline::from(route.waypoints().clone())
        };

        Ok(polyline)
    }
}
