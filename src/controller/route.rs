use actix_web::{get, web, HttpResponse, Result, Scope};

use crate::domain::route::{Route, RouteRepository};
use crate::domain::types::RouteId;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::Json;
use once_cell::sync::Lazy;

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

    pub async fn post(&self, route: Json<Route>) -> Result<HttpResponse> {
        self.repository.register(&route)?;

        Ok(HttpResponse::Created().finish())
    }
}

pub trait BuildService<S: HttpServiceFactory + 'static> {
    fn build_service(self) -> S;
}

impl<R: RouteRepository> BuildService<Scope> for &'static Lazy<RouteController<R>> {
    fn build_service(self) -> Scope {
        web::scope("/routes")
            .service(web::resource("/{id}").route(web::get().to(move |id| self.get(id))))
    }
}
