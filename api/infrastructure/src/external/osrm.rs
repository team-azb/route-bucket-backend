use std::convert::TryInto;

use async_trait::async_trait;

use route_bucket_domain::external::RouteInterpolationApi;
use route_bucket_domain::model::route::{Coordinate, DrawingMode, Polyline, Segment};
use route_bucket_utils::{ApplicationError, ApplicationResult};

/// osrmでルート補間をするための構造体
#[derive(Default)]
pub struct OsrmApi {
    api_root: String,
}

impl OsrmApi {
    pub fn new() -> Self {
        Self {
            api_root: std::env::var("OSRM_ROOT").expect("OSRM_ROOT NOT FOUND"),
        }
    }

    async fn request(&self, service: &str, args: &str) -> ApplicationResult<serde_json::Value> {
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
    async fn correct_coordinate(
        &self,
        coord: &Coordinate,
        mode: DrawingMode,
    ) -> ApplicationResult<Coordinate> {
        match mode {
            DrawingMode::FollowRoad => self
                .request(
                    "nearest",
                    &format!("{},{}", coord.longitude().value(), coord.latitude().value()),
                )
                .await
                .map(|json| {
                    let coord = serde_json::from_value::<Vec<f64>>(
                        json["waypoints"][0]["location"].clone(),
                    )
                    .unwrap();
                    Coordinate::new(coord[1], coord[0]).unwrap()
                }),
            DrawingMode::Freehand => Ok(coord.clone()),
        }
    }

    async fn interpolate(&self, seg: &mut Segment) -> ApplicationResult<()> {
        let mut points = Vec::new();
        match *seg.mode() {
            DrawingMode::FollowRoad => {
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
                    serde_json::from_value::<Polyline>(json["routes"][0]["geometry"].clone())
                        .unwrap();

                points = polyline.try_into()?;
            }
            DrawingMode::Freehand => (),
        }
        seg.set_points(points)
    }
}
