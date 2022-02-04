use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use getset::Getters;
use route_bucket_domain::model::{
    route::{Distance, Elevation, RouteId, RouteInfo},
    user::UserId,
};
use route_bucket_utils::ApplicationResult;

/// ルートのdto構造体
#[derive(sqlx::FromRow, Getters)]
#[get = "pub"]
pub struct RouteDto {
    id: String,
    name: String,
    owner_id: String,
    operation_pos: u32,
    ascent_elevation_gain: u32,
    descent_elevation_gain: u32,
    total_distance: f64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl RouteDto {
    pub fn into_model(self) -> ApplicationResult<RouteInfo> {
        let Self {
            id,
            name,
            owner_id,
            operation_pos,
            ascent_elevation_gain,
            descent_elevation_gain,
            total_distance,
            created_at,
            updated_at,
        } = self;
        Ok(RouteInfo::from((
            RouteId::from_string(id),
            name,
            UserId::from(owner_id),
            operation_pos as usize,
            Elevation::try_from(ascent_elevation_gain as i32)?,
            Elevation::try_from(descent_elevation_gain as i32)?,
            Distance::try_from(total_distance)?,
            created_at,
            updated_at,
        )))
    }

    pub fn from_model(route_info: &RouteInfo) -> ApplicationResult<RouteDto> {
        Ok(RouteDto {
            id: route_info.id().to_string(),
            name: route_info.name().clone(),
            owner_id: route_info.owner_id().to_string(),
            operation_pos: *route_info.op_num() as u32,
            ascent_elevation_gain: route_info.ascent_elevation_gain().value() as u32,
            descent_elevation_gain: route_info.descent_elevation_gain().value() as u32,
            total_distance: route_info.total_distance().value(),
            created_at: *route_info.created_at(),
            updated_at: *route_info.updated_at(),
        })
    }
}
