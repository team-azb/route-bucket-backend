use actix_web::{dev, web, HttpResponse, Result, Scope};
use once_cell::sync::Lazy;

use crate::domain::route::RouteRepository;
use crate::domain::types::RouteId;
use crate::usecase::route::{NewPointRequest, RouteCreateRequest, RouteUseCase};

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

    async fn patch_new_point(
        &self,
        path_params: web::Path<(RouteId, usize)>,
        req: web::Json<NewPointRequest>,
        op_code: &str,
    ) -> Result<HttpResponse> {
        let (id, pos) = path_params.into_inner();
        Ok(HttpResponse::Ok().json(self.usecase.edit(
            op_code,
            &id,
            Some(pos),
            Some(req.coord().clone()),
        )?))
    }

    async fn patch_remove(&self, path_params: web::Path<(RouteId, usize)>) -> Result<HttpResponse> {
        let (id, pos) = path_params.into_inner();
        Ok(HttpResponse::Ok().json(self.usecase.edit("rm", &id, Some(pos), None)?))
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
                web::resource("/{id}/add/{pos}").route(
                    web::patch().to(move |path, req| self.patch_new_point(path, req, "add")),
                ),
            )
            .service(
                web::resource("/{id}/remove/{pos}")
                    .route(web::patch().to(move |path| self.patch_remove(path))),
            )
            .service(
                web::resource("/{id}/move/{pos}")
                    .route(web::patch().to(move |path, req| self.patch_new_point(path, req, "mv"))),
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
