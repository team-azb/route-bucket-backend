pub use types::{Distance, Elevation, Latitude, Longitude, Polyline};

pub mod route;
pub(crate) mod types;

#[cfg(feature = "fixtures")]
pub mod fixtures {
    pub use super::route::bounding_box::tests::BoundingBoxFixture;
    pub use super::route::coordinate::tests::CoordinateFixtures;
    pub use super::route::operation::tests::OperationFixtures;
    pub use super::route::route_gpx::tests::RouteGpxFixtures;
    pub use super::route::route_info::tests::RouteInfoFixtures;
    pub use super::route::segment_list::tests::{SegmentFixtures, SegmentListFixture};
    pub use super::route::tests::RouteFixtures;
}
