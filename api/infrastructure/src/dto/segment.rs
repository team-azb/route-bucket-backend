use std::convert::TryFrom;

use getset::Getters;

use route_bucket_domain::model::{Polyline, RouteId, Segment};
use route_bucket_utils::ApplicationResult;

/// 座標のdto構造体
#[derive(sqlx::FromRow, Getters)]
#[get = "pub"]
pub struct SegmentDto {
    route_id: String,
    index: u32,
    polyline: String,
}

impl SegmentDto {
    pub fn into_model(self) -> ApplicationResult<Segment> {
        Segment::try_from(Polyline::from(self.polyline))
    }

    pub fn from_model(
        segment: &Segment,
        route_id: &RouteId,
        index: u32,
    ) -> ApplicationResult<SegmentDto> {
        Ok(SegmentDto {
            route_id: route_id.to_string(),
            index: index,
            polyline: Polyline::from(segment.points().clone()).into(),
        })
    }
}
