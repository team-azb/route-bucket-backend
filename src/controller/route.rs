use actix_web::{dev, web, HttpResponse, Result, Scope};
use once_cell::sync::Lazy;

use crate::domain::model::linestring::ElevationApi;
use crate::domain::model::operation::OperationRepository;
use crate::domain::model::route::{RouteInterpolationApi, RouteRepository};
use crate::domain::model::types::RouteId;
use crate::usecase::route::{
    NewPointRequest, RouteCreateRequest, RouteRenameRequest, RouteUseCase,
};

pub struct RouteController<R, O, I, E> {
    usecase: RouteUseCase<R, O, I, E>,
}

impl<R, I, O, E> RouteController<R, O, I, E>
where
    R: RouteRepository,
    O: OperationRepository,
    I: RouteInterpolationApi,
    E: ElevationApi,
{
    pub fn new(usecase: RouteUseCase<R, O, I, E>) -> Self {
        Self { usecase }
    }

    async fn get(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.find(id.as_ref())?))
    }

    async fn get_all(&self) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.find_all()?))
    }

    async fn post(&self, req: web::Json<RouteCreateRequest>) -> Result<HttpResponse> {
        Ok(HttpResponse::Created().json(self.usecase.create(&req)?))
    }

    async fn patch_rename(
        &self,
        id: web::Path<RouteId>,
        req: web::Json<RouteRenameRequest>,
    ) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.rename(&id, &req)?))
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

    async fn delete(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        self.usecase.delete(&id)?;
        Ok(HttpResponse::Ok().finish())
    }
}

pub trait BuildService<S: dev::HttpServiceFactory + 'static> {
    fn build_service(self) -> S;
}

impl<R, O, I, E> BuildService<Scope> for &'static Lazy<RouteController<R, O, I, E>>
where
    R: RouteRepository,
    O: OperationRepository,
    I: RouteInterpolationApi,
    E: ElevationApi,
{
    fn build_service(self) -> Scope {
        // TODO: /の過不足は許容する ex) "/{id}/"
        web::scope("/routes")
            .service(
                web::resource("/")
                    .route(web::get().to(move || self.get_all()))
                    .route(web::post().to(move |req| self.post(req))),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(move |id| self.get(id)))
                    .route(web::delete().to(move |id| self.delete(id))),
            )
            .service(
                web::resource("/{id}/rename/")
                    .route(web::patch().to(move |id, req| self.patch_rename(id, req))),
            )
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
