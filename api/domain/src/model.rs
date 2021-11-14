pub mod route;
pub mod types;
pub mod user;

#[cfg(feature = "fixtures")]
pub mod fixtures {
    pub mod route {
        pub use crate::model::route::bounding_box::tests::BoundingBoxFixture;
        pub use crate::model::route::coordinate::tests::CoordinateFixtures;
        pub use crate::model::route::operation::tests::OperationFixtures;
        pub use crate::model::route::route_gpx::tests::RouteGpxFixtures;
        pub use crate::model::route::route_info::tests::RouteInfoFixtures;
        pub use crate::model::route::segment_list::tests::{SegmentFixtures, SegmentListFixture};
        pub use crate::model::route::tests::RouteFixtures;
    }
}
