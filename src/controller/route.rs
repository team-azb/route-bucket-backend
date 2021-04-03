use actix_web::{dev, web, HttpResponse, Result, Scope};
use once_cell::sync::Lazy;

use crate::domain::route::RouteRepository;
use crate::domain::types::RouteId;
use crate::usecase::route::{RouteCreateRequest, RouteUseCase};

pub struct RouteController<R: RouteRepository> {
    usecase: RouteUseCase<R>,
}

impl<R: RouteRepository> RouteController<R> {
    pub fn new(usecase: RouteUseCase<R>) -> RouteController<R> {
        RouteController { usecase }
    }

    async fn get(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.find(id.as_ref())?))
    }

    async fn post(&self, req: web::Json<RouteCreateRequest>) -> Result<HttpResponse> {
        Ok(HttpResponse::Created().json(self.usecase.create(&req)?))
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
