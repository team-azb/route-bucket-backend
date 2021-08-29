// TODO: この辺をmodel/route.rsに移動して、外からもmodel::route::でアクセスさせるようにする
//     : あるいは、lib.rsでpub mod routeとuserを切る
pub use route::coordinate::Coordinate;
pub use route::operation::{Operation, OperationType};
pub use route::route_gpx::RouteGpx;
pub use route::route_info::RouteInfo;
pub use route::segment_list::{Segment, SegmentList};
pub use route::Route;
pub use types::{Distance, Elevation, Latitude, Longitude, Polyline, RouteId, UserId};
pub use user::User;

pub(crate) mod route;
pub(crate) mod types;
pub mod user;
