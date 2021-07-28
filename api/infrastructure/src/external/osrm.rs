use async_trait::async_trait;
use route_bucket_domain::model::{Coordinate, Polyline, Segment};
use route_bucket_domain::repository::RouteInterpolationApi;
use route_bucket_utils::{ApplicationError, ApplicationResult};
use std::convert::TryInto;

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

    async fn request(&self, service: &str, args: &String) -> ApplicationResult<serde_json::Value> {
        let url_str =
            format!("{}/{}/v1/bike/{}", self.api_root, service, args).replace("\\", "%5C");
        let url = reqwest::Url::parse(&url_str).map_err(|err| {
            ApplicationError::ExternalError(format!(
                "Failed to parse OSRM URL: {} ({})",
                url_str, err
            ))
        })?;
        let resp = reqwest::get(url.clone())
            .await
            .map_err(|_| ApplicationError::ExternalError(format!("Failed to request {}", url)))?;

        if resp.status().is_success() {
            resp.json::<serde_json::Value>().await.map_err(|_| {
                ApplicationError::ExternalError("Failed to parse response json".into())
            })
        } else {
            Err(ApplicationError::ExternalError(format!(
                "Got unsuccessful status {} from OSRM (url: {}, body: {})",
                resp.status(),
                resp.url().clone(),
                resp.json::<serde_json::Value>().await.unwrap()
            )))
        }
    }
}

#[async_trait]
impl RouteInterpolationApi for OsrmApi {
    async fn correct_coordinate(&self, coord: &Coordinate) -> ApplicationResult<Coordinate> {
        self.request(
            "nearest",
            &format!("{},{}", coord.longitude().value(), coord.latitude().value()),
        )
        .await
        .map(|json| {
            let coord =
                serde_json::from_value::<Vec<f64>>(json["waypoints"][0]["location"].clone())
                    .unwrap();
            Coordinate::new(coord[1], coord[0]).unwrap()
        })
    }

    async fn interpolate(&self, seg: &mut Segment) -> ApplicationResult<()> {
        let json = self
            .request(
                "route",
                &format!(
                    "polyline({})?overview=full",
                    String::from(Polyline::from(vec![
                        seg.start().clone(),
                        seg.goal().clone()
                    ]))
                ),
            )
            .await?;

        let polyline =
            serde_json::from_value::<Polyline>(json["routes"][0]["geometry"].clone()).unwrap();

        seg.set_points(polyline.try_into()?)
    }
}
