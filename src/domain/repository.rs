use crate::domain::model::linestring::Coordinate;
use crate::domain::model::operation::Operation;
use crate::domain::model::route::RouteInfo;
use crate::domain::model::segment::{Segment, SegmentList};
use crate::domain::model::types::{Elevation, RouteId};
use crate::utils::error::ApplicationResult;
use std::ops::Range;

pub trait RouteRepository {
    fn find(&self, id: &RouteId) -> ApplicationResult<RouteInfo>;

    fn find_all(&self) -> ApplicationResult<Vec<RouteInfo>>;

    fn register(&self, route_info: &RouteInfo) -> ApplicationResult<()>;

    fn update(&self, route_info: &RouteInfo) -> ApplicationResult<()>;

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

pub trait SegmentRepository {
    fn update(&self, route_id: &RouteId, pos: u32, seg: &Segment) -> ApplicationResult<()>;

    fn insert(&self, route_id: &RouteId, pos: u32, seg: &Segment) -> ApplicationResult<()>;

    fn delete(&self, route_id: &RouteId, pos: u32) -> ApplicationResult<()>;

    fn find_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<SegmentList>;

    fn insert_by_route_id(
        &self,
        route_id: &RouteId,
        seg_list: &SegmentList,
    ) -> ApplicationResult<()>;

    fn delete_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<()>;

    fn delete_by_route_id_and_range(
        &self,
        route_id: &RouteId,
        range: Range<u32>,
    ) -> ApplicationResult<()>;
}

pub trait RouteInterpolationApi {
    fn correct_coordinate(&self, coord: &Coordinate) -> ApplicationResult<Coordinate>;

    fn interpolate(&self, from: Coordinate, to: Coordinate) -> ApplicationResult<Segment>;
}

pub trait ElevationApi {
    fn get_elevation(&self, coord: &Coordinate) -> ApplicationResult<Option<Elevation>>;
}
