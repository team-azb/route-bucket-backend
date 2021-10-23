// TODO: この辺をmodel/route.rsに移動して、外からもmodel::route::でアクセスさせるようにする
pub use route::bounding_box::BoundingBox;
pub use route::coordinate::Coordinate;
pub use route::operation::{Operation, OperationType, SegmentTemplate};
pub use route::route_gpx::RouteGpx;
pub use route::route_info::RouteInfo;
pub use route::segment_list::{DrawingMode, Segment, SegmentList};
pub use route::Route;
pub use types::{Distance, Elevation, Latitude, Longitude, OperationId, Polyline, RouteId};

pub(crate) mod route;
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
