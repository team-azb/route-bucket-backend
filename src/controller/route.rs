use actix_web::{dev, web, HttpResponse, Result, Scope};
use getset::Getters;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::domain::route::{Route, RouteRepository};
use crate::domain::types::RouteId;

#[derive(Clone)]
pub struct RouteController<Repository: RouteRepository> {
    repository: Repository,
}

impl<Repository: RouteRepository> RouteController<Repository> {
    pub fn new(repository: Repository) -> RouteController<Repository> {
        RouteController { repository }
    }

    pub async fn get(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        let route = self.repository.find(id.as_ref())?;

        Ok(HttpResponse::Ok().json(route))
    }

    pub async fn post(&self, req: web::Json<RouteCreateRequest>) -> Result<HttpResponse> {
        let route_id = self.repository.create(&req.name())?;

        Ok(HttpResponse::Created().json(RouteCreateResponse::new(&route_id)))
    }
}

pub trait BuildService<S: dev::HttpServiceFactory + 'static> {
    fn build_service(self) -> S;
}

impl<R: RouteRepository> BuildService<Scope> for &'static Lazy<RouteController<R>> {
    fn build_service(self) -> Scope {
        web::scope("/routes")
            .service(web::resource("/{id}").route(web::get().to(move |id| self.get(id))))
            .service(web::resource("/").route(web::post().to(move |req| self.post(req))))
    }
}

// TODO: UseCaseを作ったらそっちに移動する
/// response body for POST /routes/
#[derive(Getters, Deserialize)]
#[get = "pub"]
struct RouteCreateRequest {
    name: String,
}

/// response body for POST /routes/
#[derive(Serialize)]
struct RouteCreateResponse {
    id: RouteId,
}
impl RouteCreateResponse {
    fn new(id: &RouteId) -> Self {
        Self { id: id.clone() }
    }
}
