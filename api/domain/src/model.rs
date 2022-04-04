pub mod route;
pub mod types;
pub mod user;

#[cfg(feature = "fixtures")]
pub mod fixtures {
    pub mod route {
        pub use crate::model::route::bounding_box::tests::BoundingBoxFixture;
        pub use crate::model::route::coordinate::tests::CoordinateFixtures;
        pub use crate::model::route::route_gpx::tests::RouteGpxFixtures;
        pub use crate::model::route::route_info::tests::RouteInfoFixtures;
        pub use crate::model::route::search_query::tests::RouteSearchQueryFixtures;
        pub use crate::model::route::segment_list::tests::{
            OperationFixtures, SegmentFixtures, SegmentListFixture,
        };
        pub use crate::model::route::tests::RouteFixtures;
    }

    pub mod user {
        pub use crate::model::user::tests::{UserFixtures, UserIdFixtures};
    }
}
