use std::ops::Range;

use async_trait::async_trait;
use futures::FutureExt;
use itertools::Itertools;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::pool::PoolConnection;
use sqlx::{MySql, MySqlPool};

use crate::domain::model::operation::Operation;
use crate::domain::model::route::{Route, RouteInfo};
use crate::domain::model::segment::{Segment, SegmentList};
use crate::domain::model::types::RouteId;
use crate::domain::repository::{Connection, Repository, RouteRepository};
use crate::infrastructure::dto::operation::OperationDto;
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::dto::segment::SegmentDto;
use crate::infrastructure::repository;
use crate::utils::error::{ApplicationError, ApplicationResult};

// NOTE: MySqlPoolを共有したくなったら、Arcで囲めば良さそう
pub struct RouteRepositoryMySql(MySqlPool);

impl RouteRepositoryMySql {
    pub async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");
        MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .map(|res| res.map(Self))
            .await
            .unwrap()
    }

    // TODO: この辺を、テーブル名とかWHERE以下を変数にして関数にまとめる
    async fn shift_segments(
        id: &RouteId,
        start_pos: u32,
        to_right: bool,
        width: u32,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        let query = format!(
            r"
            UPDATE segments 
            SET `index` = `index` {} ? 
            WHERE route_id = ? AND `index` >= ?
            ORDER BY `route_id` {}
            ",
            if to_right { "+" } else { "-" },
            if to_right { "DESC" } else { "ASC" }
        );
        sqlx::query(&query)
            .bind(width)
            .bind(id.to_string())
            .bind(start_pos)
            .execute(conn)
            .await
            .map_err(repository::gen_err_mapper("failed to shift segments"))?;
        Ok(())
    }

    async fn find_op_list(
        id: &RouteId,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<Vec<Operation>> {
        sqlx::query_as::<_, OperationDto>(
            r"
            SELECT * FROM operations WHERE route_id = ?
            ",
        )
        .bind(id.to_string())
        // NOTE: ここで一旦collectしてしまっている, これ避けられないか
        .fetch_all(conn)
        .await
        .map_err(repository::gen_err_mapper("failed to find operations"))?
        .into_iter()
        .map(OperationDto::into_model)
        .try_collect()
    }

    async fn find_seg_list(
        id: &RouteId,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<SegmentList> {
        sqlx::query_as::<_, SegmentDto>(
            r"
            SELECT *
            FROM segments 
            WHERE route_id = ? 
            ORDER BY route_id, `index`
            ",
        )
        .bind(id.to_string())
        .fetch_all(conn)
        .await
        .map_err(repository::gen_err_mapper("failed to find segments"))?
        .into_iter()
        .map(SegmentDto::into_model)
        .collect::<ApplicationResult<Vec<_>>>()
        .map(SegmentList::from)
    }

    async fn insert_operation(
        dto: &OperationDto,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        sqlx::query(
            r"
            INSERT INTO operations
            VALUES (?, ?, ?, ?, ?)
            ",
        )
        .bind(dto.route_id())
        .bind(dto.index())
        .bind(dto.code())
        .bind(dto.pos())
        .bind(dto.polyline())
        .execute(conn)
        .await
        .map_err(repository::gen_err_mapper("failed to insert Operation"))?;

        Ok(())
    }

    async fn insert_segment(
        id: &RouteId,
        pos: u32,
        seg: &Segment,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        let dto = SegmentDto::from_model(seg, id, pos)?;

        sqlx::query(
            r"
            INSERT INTO segments VALUES (?, ?, ?)
            ",
        )
        .bind(dto.route_id())
        .bind(dto.index())
        .bind(dto.polyline())
        .execute(conn)
        .await
        .map_err(repository::gen_err_mapper("failed to insert Segment"))?;

        Ok(())
    }

    async fn delete_operations_by_start(
        route_id: &RouteId,
        start_pos: u32,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        sqlx::query(
            r"
            DELETE FROM operations WHERE `route_id` = ? AND `index` >= ?
            ",
        )
        .bind(route_id.to_string())
        .bind(start_pos)
        .execute(conn)
        .await
        .map_err(repository::gen_err_mapper("failed to insert Operation"))?;

        Ok(())
    }

    async fn delete_segments_by_range(
        id: &RouteId,
        range: Range<u32>,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        sqlx::query(
            r"
            DELETE FROM segments 
            WHERE 
                  `route_id` = ? AND 
                  ? <= `index` AND `index` < ?
            ",
        )
        .bind(id.to_string())
        .bind(range.start)
        .bind(range.end)
        .execute(conn)
        .await
        .map_err(repository::gen_err_mapper(
            "failed to delete segments by range",
        ))?;

        Ok(())
    }

    async fn delete_by_route_id(
        id: &RouteId,
        table_name: &str,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        let id_name = match table_name {
            "routes" => Ok("id"),
            "operations" | "segments" => Ok("route_id"),
            _ => Err(ApplicationError::DataBaseError(format!(
                "Invalid table_name {} for delete_by_route_id",
                table_name
            ))),
        }?;
        let query = format!("DELETE FROM {} WHERE `{}` = ?", table_name, id_name);

        sqlx::query(&query)
            .bind(id.to_string())
            .execute(conn)
            .await
            .map_err(repository::gen_err_mapper("failed to delete route"))?;

        Ok(())
    }
}

#[async_trait]
impl Repository for RouteRepositoryMySql {
    type Connection = PoolConnection<MySql>;

    async fn get_connection(&self) -> ApplicationResult<Self::Connection> {
        self.0
            .acquire()
            .await
            .map_err(repository::gen_err_mapper("failed to get connection"))
    }
}

#[async_trait]
impl RouteRepository for RouteRepositoryMySql {
    async fn find(
        &self,
        id: &RouteId,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<Route> {
        conn.begin_transaction().await?;

        let info = self.find_info(id, conn).await?;
        let op_list = Self::find_op_list(id, conn).await?;
        let seg_list = Self::find_seg_list(id, conn).await?;

        conn.commit_transaction().await?;

        Ok(Route::new(info, op_list, seg_list))
    }

    async fn find_info(
        &self,
        id: &RouteId,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<RouteInfo> {
        sqlx::query_as::<_, RouteDto>(
            r"
            SELECT * FROM routes WHERE id = ?
            ",
        )
        .bind(id.to_string())
        .fetch_one(conn)
        .await
        .map_err(repository::gen_err_mapper("failed to find info"))?
        .into_model()
    }

    async fn find_all_infos(
        &self,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<Vec<RouteInfo>> {
        sqlx::query_as::<_, RouteDto>(
            r"
            SELECT * FROM routes
            ",
        )
        .fetch_all(conn)
        .await
        .map_err(repository::gen_err_mapper("failed to find infos"))?
        .into_iter()
        .map(RouteDto::into_model)
        .collect::<ApplicationResult<Vec<_>>>()
    }

    async fn insert_info(
        &self,
        info: &RouteInfo,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        let dto = RouteDto::from_model(info)?;

        sqlx::query(
            r"
            INSERT INTO routes VALUES (?, ?, ?)
            ",
        )
        .bind(dto.id())
        .bind(dto.name())
        .bind(dto.operation_pos())
        .execute(conn)
        .await
        .map_err(repository::gen_err_mapper("failed to insert RouteInfo"))?;
        Ok(())
    }

    async fn insert_and_shift_segments(
        &self,
        id: &RouteId,
        pos: u32,
        seg: &Segment,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        conn.begin_transaction().await?;

        Self::shift_segments(id, pos, true, 1, conn).await?;
        Self::insert_segment(id, pos, seg, conn).await?;

        conn.commit_transaction().await?;

        Ok(())
    }

    async fn insert_and_truncate_operations(
        &self,
        id: &RouteId,
        pos: u32,
        op: &Operation,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        let dto = OperationDto::from_model(op, id, pos)?;

        conn.begin_transaction().await?;

        Self::delete_operations_by_start(id, pos, conn).await?;
        Self::insert_operation(&dto, conn).await?;

        conn.commit_transaction().await?;

        Ok(())
    }

    async fn update_info(
        &self,
        info: &RouteInfo,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        let dto = RouteDto::from_model(info)?;

        sqlx::query(
            r"
            UPDATE routes
            SET name = ?, operation_pos = ?
            WHERE id = ?
            ",
        )
        .bind(dto.name())
        .bind(dto.operation_pos())
        .bind(dto.id())
        .execute(conn)
        .await
        .map_err(repository::gen_err_mapper("failed to update Operation"))?;

        Ok(())
    }

    async fn delete(
        &self,
        id: &RouteId,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        Self::delete_by_route_id(id, "routes", conn).await?;
        Self::delete_by_route_id(id, "operations", conn).await?;
        Self::delete_by_route_id(id, "segments", conn).await?;

        Ok(())
    }

    async fn delete_and_shift_segments_by_range(
        &self,
        id: &RouteId,
        range: Range<u32>,
        conn: &mut PoolConnection<MySql>,
    ) -> ApplicationResult<()> {
        conn.begin_transaction().await?;

        Self::delete_segments_by_range(id, range.clone(), conn).await?;
        Self::shift_segments(id, range.end, false, range.end - range.start, conn).await?;

        conn.commit_transaction().await?;

        Ok(())
    }
}
