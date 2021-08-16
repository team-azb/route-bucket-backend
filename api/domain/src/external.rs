use async_trait::async_trait;

use route_bucket_utils::ApplicationResult;

use crate::model::{Coordinate, Elevation, Segment};

#[async_trait]
pub trait RouteInterpolationApi: Send + Sync {
    async fn correct_coordinate(&self, coord: &Coordinate) -> ApplicationResult<Coordinate>;

    async fn interpolate(&self, seg: &mut Segment) -> ApplicationResult<()>;
}

pub trait ElevationApi: Send + Sync {
    fn get_elevation(&self, coord: &Coordinate) -> ApplicationResult<Option<Elevation>>;
}
