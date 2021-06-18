use crate::domain::model::linestring::Coordinate;
use crate::domain::model::operation::Operation;
use crate::domain::model::route::Route;
use crate::domain::model::types::{Elevation, Polyline, RouteId};
use crate::utils::error::ApplicationResult;

pub trait RouteRepository {
    fn find(&self, id: &RouteId) -> ApplicationResult<Route>;

    fn find_all(&self) -> ApplicationResult<Vec<Route>>;

    fn register(&self, route: &Route) -> ApplicationResult<()>;

    fn update(&self, route: &Route) -> ApplicationResult<()>;

    fn delete(&self, id: &RouteId) -> ApplicationResult<()>;
}

pub trait OperationRepository {
    fn find_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<Vec<Operation>>;

    fn update_by_route_id(
        &self,
        route_id: &RouteId,
        op_list: &Vec<Operation>,
    ) -> ApplicationResult<()>;

    fn delete_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<()>;
}

pub trait RouteInterpolationApi {
    fn correct_coordinate(&self, coord: &Coordinate) -> ApplicationResult<Coordinate>;

    fn interpolate(&self, route: &Route) -> ApplicationResult<Polyline>;
}

pub trait ElevationApi {
    fn get_elevation(&self, coord: &Coordinate) -> ApplicationResult<Option<Elevation>>;
}
