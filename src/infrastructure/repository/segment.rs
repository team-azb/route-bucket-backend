use diesel::associations::HasTable;
use diesel::{ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl};

use crate::domain::model::segment::{Segment, SegmentList};
use crate::domain::model::types::RouteId;
use crate::domain::repository::SegmentRepository;
use crate::infrastructure::dto::segment::SegmentDto;
use crate::infrastructure::repository::connection_pool::MysqlConnectionPool;
use crate::infrastructure::schema::segments;
use crate::utils::error::{ApplicationError, ApplicationResult};
use diesel::r2d2::{ConnectionManager, PooledConnection};

pub struct SegmentRepositoryMysql {
    pool: MysqlConnectionPool,
}

impl SegmentRepositoryMysql {
    pub fn new() -> Self {
        Self {
            pool: MysqlConnectionPool::new(),
        }
    }

    fn shift_segments(
        &self,
        route_id: &RouteId,
        start_pos: u32,
        right_shift: bool,
        conn: &PooledConnection<ConnectionManager<MysqlConnection>>,
    ) -> ApplicationResult<usize> {
        let src = SegmentDto::table()
            .filter(segments::route_id.eq(route_id.to_string()))
            .filter(segments::index.ge(start_pos as i32));
        let err_msg = format!(
            "Failed to update Segments that belong to Route {}",
            route_id.to_string()
        );

        if right_shift {
            diesel::update(src)
                .set(segments::index.eq(segments::index + 1))
                .execute(conn)
                .map_err(|_| ApplicationError::DataBaseError(err_msg))
        } else {
            diesel::update(src)
                .set(segments::index.eq(segments::index - 1))
                .execute(conn)
                .map_err(|_| ApplicationError::DataBaseError(err_msg))
        }
    }
}

impl SegmentRepository for SegmentRepositoryMysql {
    fn update(&self, route_id: &RouteId, pos: u32, seg: &Segment) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        let seg_dto = SegmentDto::from_model(seg, route_id, pos)?;

        diesel::update(
            SegmentDto::table()
                .filter(segments::route_id.eq(route_id.to_string()))
                .filter(segments::index.eq(pos as i32)),
        )
        .set(seg_dto)
        .execute(&conn)
        .map_err(|_| {
            ApplicationError::DataBaseError(format!(
                "Failed to update Segments that belong to Route {}",
                route_id.to_string()
            ))
        })?;

        Ok(())
    }

    fn insert(&self, route_id: &RouteId, pos: u32, seg: &Segment) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        self.shift_segments(route_id, pos, true, &conn)?;

        let seg_dto = SegmentDto::from_model(seg, route_id, pos)?;

        diesel::insert_into(SegmentDto::table())
            .values(seg_dto)
            .execute(&conn)
            .map_err(|_| {
                ApplicationError::DataBaseError(format!(
                    "Failed to insert Segments to {}",
                    route_id.to_string()
                ))
            })?;

        Ok(())
    }

    fn delete(&self, route_id: &RouteId, pos: u32) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        diesel::delete(
            SegmentDto::table()
                .filter(segments::route_id.eq(route_id.to_string()))
                .filter(segments::index.eq(pos as i32)),
        )
        .execute(&conn)
        .map_err(|_| {
            ApplicationError::DataBaseError(format!(
                "Failed to delete Segments that belong to Route {}",
                route_id.to_string()
            ))
        })?;

        self.shift_segments(route_id, pos, false, &conn)?;

        Ok(())
    }

    fn find_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<SegmentList> {
        let conn = self.pool.get_connection()?;

        Ok(SegmentDto::table()
            .filter(segments::route_id.eq(route_id.to_string()))
            .load::<SegmentDto>(&conn)
            .map_err(|_| ApplicationError::DataBaseError("Failed to find Segments".into()))?
            .iter()
            .map(SegmentDto::to_model)
            .collect::<ApplicationResult<Vec<Segment>>>()?
            .into())
    }

    fn insert_by_route_id(
        &self,
        route_id: &RouteId,
        seg_list: &SegmentList,
    ) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        let seg_dtos = seg_list
            .iter()
            .enumerate()
            .map(|(i, seg)| SegmentDto::from_model(seg, route_id, i as u32))
            .collect::<ApplicationResult<Vec<_>>>()?;

        diesel::insert_into(SegmentDto::table())
            .values(seg_dtos)
            .execute(&conn)
            .map_err(|_| {
                ApplicationError::DataBaseError(format!(
                    "Failed to insert Segments to {}",
                    route_id.to_string()
                ))
            })?;

        Ok(())
    }

    fn delete_by_route_id(&self, route_id: &RouteId) -> ApplicationResult<()> {
        let conn = self.pool.get_connection()?;

        diesel::delete(SegmentDto::table().filter(segments::route_id.eq(route_id.to_string())))
            .execute(&conn)
            .map_err(|_| {
                ApplicationError::DataBaseError(format!(
                    "Failed to delete Segments that belong to Route {}",
                    route_id.to_string()
                ))
            })?;

        Ok(())
    }
}
