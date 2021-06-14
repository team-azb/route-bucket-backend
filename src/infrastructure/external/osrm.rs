use crate::domain::model::linestring::Coordinate;
use crate::domain::model::route::{Route, RouteInterpolationApi};
use crate::domain::model::types::Polyline;
use crate::utils::error::{ApplicationError, ApplicationResult};

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

    fn request(
        &self,
        service: &str,
        args: &String,
    ) -> ApplicationResult<reqwest::blocking::Response> {
        let target_url = format!("{}/{}/v1/bike/{}", self.api_root, service, args);
        reqwest::blocking::get(target_url.clone()).map_err(|_| {
            ApplicationError::ExternalError(format!("Failed to request {}", target_url))
        })
    }

    fn resp_to_json(resp: reqwest::blocking::Response) -> ApplicationResult<serde_json::Value> {
        let status = resp.status();
        let url = resp.url().clone().into_string();
        status
            .is_success()
            .then(|| resp.json::<serde_json::Value>().unwrap())
            .ok_or(ApplicationError::ExternalError(format!(
                "Got Unsuccessful status {} from {}",
                status, url
            )))
    }
}

impl RouteInterpolationApi for OsrmApi {
    fn correct_coordinate(&self, coord: &Coordinate) -> ApplicationResult<Coordinate> {
        let resp = self.request(
            "nearest",
            &format!("{},{}", coord.longitude().value(), coord.latitude().value()),
        )?;

        Self::resp_to_json(resp).map(|json| {
            let coord =
                serde_json::from_value::<Vec<f64>>(json["waypoints"][0]["location"].clone())
                    .unwrap();
            Coordinate::new(coord[1], coord[0]).unwrap()
        })
    }

    fn interpolate(&self, route: &Route) -> ApplicationResult<Polyline> {
        let resp = self.request(
            "route",
            &format!(
                "polyline({})?overview=full",
                String::from(Polyline::from(route.waypoints().clone()))
            ),
        )?;

        Ok(
            Self::resp_to_json(resp).map_or(Polyline::from(route.waypoints().clone()), |json| {
                Polyline::from(
                    serde_json::from_value::<String>(json["routes"][0]["geometry"].clone())
                        .unwrap(),
                )
            }),
        )
    }
}
