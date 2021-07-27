use getset::Getters;

use route_bucket_domain::model::{RouteId, RouteInfo};
use route_bucket_utils::ApplicationResult;

/// ルートのdto構造体
#[derive(sqlx::FromRow, Getters)]
#[get = "pub"]
pub struct RouteDto {
    id: String,
    name: String,
    operation_pos: u32,
}

impl RouteDto {
    pub fn into_model(self) -> ApplicationResult<RouteInfo> {
        let Self {
            id,
            name,
            operation_pos,
        } = self;
        Ok(RouteInfo::new(
            RouteId::from_string(id),
            &name,
            operation_pos as usize,
        ))
    }

    pub fn from_model(route_info: &RouteInfo) -> ApplicationResult<RouteDto> {
        Ok(RouteDto {
            id: route_info.id().to_string(),
            name: route_info.name().clone(),
            operation_pos: *route_info.op_num() as u32,
        })
    }
}
