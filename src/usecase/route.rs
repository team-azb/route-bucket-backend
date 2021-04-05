use getset::Getters;
use serde::{Deserialize, Serialize};

use crate::domain::polyline::Polyline;
use crate::domain::route::{Route, RouteRepository};
use crate::domain::types::RouteId;
use crate::utils::error::ApplicationResult;

pub struct RouteUseCase<R: RouteRepository> {
    repository: R,
}

impl<R: RouteRepository> RouteUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn find(&self, route_id: &RouteId) -> ApplicationResult<Route> {
        self.repository.find(route_id)
    }

    pub fn create(&self, req: &RouteCreateRequest) -> ApplicationResult<RouteCreateResponse> {
        let route = Route::new(RouteId::new(), req.name(), Polyline::new());

        self.repository.register(&route)?;
        Ok(RouteCreateResponse::new(route.id()))
    }
}

#[derive(Getters, Deserialize)]
#[get = "pub"]
pub struct RouteCreateRequest {
    name: String,
}

#[derive(Serialize)]
pub struct RouteCreateResponse {
    id: RouteId,
}
impl RouteCreateResponse {
    pub fn new(id: &RouteId) -> Self {
        Self { id: id.clone() }
    }
}
