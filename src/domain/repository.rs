use std::ops::Range;

use async_trait::async_trait;

use crate::domain::model::coordinate::Coordinate;
use crate::domain::model::operation::Operation;
use crate::domain::model::route::{Route, RouteInfo};
use crate::domain::model::segment::Segment;
use crate::domain::model::types::{Elevation, RouteId};
use crate::utils::error::ApplicationResult;
use futures::future::BoxFuture;
use std::future::Future;

#[async_trait]
pub trait Connection: Sync {
    async fn begin_transaction(&self) -> ApplicationResult<()>;
    async fn commit_transaction(&self) -> ApplicationResult<()>;
    async fn rollback_transaction(&self) -> ApplicationResult<()>;

    // Fellow rustaceans helped me to make this work.
    // https://users.rust-lang.org/t/lifetime-may-not-live-long-enough-for-an-async-closure/62489
    // https://users.rust-lang.org/t/how-to-use-self-inside-an-async-closure/62540
    async fn transaction<'a, T, F>(&'a self, f: F) -> ApplicationResult<T>
    where
        T: Send,
        F: FnOnce(&'a Self) -> BoxFuture<'a, ApplicationResult<T>> + Send,
    {
        self.begin_transaction().await?;
        let result = f(self).await;

        if result.is_ok() {
            self.commit_transaction().await?;
        } else {
            self.rollback_transaction().await?;
        }
        result
    }
}

#[async_trait]
pub trait Repository: Send + Sync {
    type Connection: Connection + Send;

    async fn get_connection(&self) -> ApplicationResult<Self::Connection>;

    async fn begin_transaction(&self) -> ApplicationResult<Self::Connection> {
        let conn = self.get_connection().await?;
        conn.begin_transaction().await?;
        Ok(conn)
    }
}

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

#[async_trait]
pub trait RouteInterpolationApi: Send + Sync {
    async fn correct_coordinate(&self, coord: &Coordinate) -> ApplicationResult<Coordinate>;

    async fn interpolate(&self, seg: &mut Segment) -> ApplicationResult<()>;
}

pub trait ElevationApi: Send + Sync {
    fn get_elevation(&self, coord: &Coordinate) -> ApplicationResult<Option<Elevation>>;
}
