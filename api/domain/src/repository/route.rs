use std::ops::Range;

use async_trait::async_trait;

use route_bucket_utils::ApplicationResult;

use crate::model::{Operation, Route, RouteId, RouteInfo, Segment};
use crate::repository::Repository;

#[async_trait]
pub trait RouteRepository: Repository {
    // type Connection = <Self as Repository>::Connection;
    // | error[E0658]: associated type defaults are unstable
    // | see issue #29661 <https://github.com/rust-lang/rust/issues/29661> for more information

    async fn find(
        &self,
        id: &RouteId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<Route>;

    async fn find_info(
        &self,
        id: &RouteId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<RouteInfo>;

    async fn find_all_infos(&self, conn: &Self::Connection) -> ApplicationResult<Vec<RouteInfo>>;

    async fn insert_info(
        &self,
        info: &RouteInfo,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;

    async fn insert_and_shift_segments(
        &self,
        id: &RouteId,
        pos: u32,
        seg: &Segment,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;

    async fn insert_and_truncate_operations(
        &self,
        id: &RouteId,
        pos: u32,
        op: &Operation,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;

    async fn update_info(
        &self,
        info: &RouteInfo,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;

    async fn delete(
        &self,
        id: &RouteId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;

    async fn delete_and_shift_segments_by_range(
        &self,
        id: &RouteId,
        range: Range<u32>,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;
}

pub trait CallRouteRepository {
    type RouteRepository: RouteRepository;

    fn route_repository(&self) -> &Self::RouteRepository;
}
