use std::convert::TryFrom;

use crate::domain::model::segment::Segment;
use crate::domain::model::types::{Polyline, RouteId};
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::schema::segments;
use crate::utils::error::ApplicationResult;

/// 座標のdto構造体
#[derive(Identifiable, Queryable, Insertable, Associations, Debug, AsChangeset)]
#[table_name = "segments"]
#[primary_key(route_id, index)]
#[belongs_to(RouteDto, foreign_key = "route_id")]
pub struct SegmentDto {
    route_id: String,
    // UNSIGNEDにすると、なぜかdieselでインクリメントのアップデートができない
    // 参考：https://github.com/diesel-rs/diesel/issues/2382
    index: i32,
    polyline: String,
}

impl SegmentDto {
    pub fn to_model(&self) -> ApplicationResult<Segment> {
        Segment::try_from(Polyline::from(self.polyline.clone()))
    }

    pub fn from_model(
        segment: &Segment,
        route_id: &RouteId,
        index: u32,
    ) -> ApplicationResult<SegmentDto> {
        Ok(SegmentDto {
            route_id: route_id.to_string(),
            index: index as i32,
            polyline: Polyline::from(segment.points().clone()).into(),
        })
    }
}
