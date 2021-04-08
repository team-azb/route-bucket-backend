use actix_web::{dev, web, HttpResponse, Result, Scope};
use once_cell::sync::Lazy;

use crate::domain::operation_history::Operation;
use crate::domain::route::RouteRepository;
use crate::domain::types::RouteId;
use crate::usecase::route::{AddPointRequest, RouteCreateRequest, RouteUseCase};

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

    async fn patch_add(
        &self,
        id: web::Path<RouteId>,
        pos: web::Path<usize>,
        req: web::Json<AddPointRequest>,
    ) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.edit(
            "add",
            &id,
            Some(*pos),
            Some(req.coord().clone()),
        )?))
    }

    async fn patch_remove(
        &self,
        id: web::Path<RouteId>,
        pos: web::Path<usize>,
    ) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.edit("rm", &id, Some(*pos), None)?))
    }

    async fn patch_clear(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.edit("clear", &id, None, None)?))
    }

    async fn patch_undo(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.migrate_history(&id, false)?))
    }

    async fn patch_redo(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.migrate_history(&id, true)?))
    }
}

pub trait BuildService<S: dev::HttpServiceFactory + 'static> {
    fn build_service(self) -> S;
}

impl<R: RouteRepository> BuildService<Scope> for &'static Lazy<RouteController<R>> {
    fn build_service(self) -> Scope {
        // TODO: /の過不足は許容する ex) "/{id}/"
        web::scope("/routes")
            .service(web::resource("/{id}").route(web::get().to(move |id| self.get(id))))
            .service(web::resource("/").route(web::post().to(move |req| self.post(req))))
            .service(
                web::resource("/{id}/add/{pos}")
                    .route(web::patch().to(move |id, pos, req| self.patch_add(id, pos, req))),
            )
            .service(
                web::resource("/{id}/remove/{pos}")
                    .route(web::patch().to(move |id, pos| self.patch_remove(id, pos))),
            )
            .service(
                web::resource("/{id}/clear/")
                    .route(web::patch().to(move |id| self.patch_clear(id))),
            )
            .service(
                web::resource("/{id}/undo/").route(web::patch().to(move |id| self.patch_undo(id))),
            )
            .service(
                web::resource("/{id}/redo/").route(web::patch().to(move |id| self.patch_redo(id))),
            )
    }
}
