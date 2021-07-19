use actix_web::{dev, http, web, HttpResponse, Result, Scope};
use once_cell::sync::Lazy;

use crate::domain::model::types::RouteId;
use crate::domain::repository::{
    ElevationApi, OperationRepository, RouteInterpolationApi, RouteRepository, SegmentRepository,
};
use crate::usecase::route::{
    NewPointRequest, RouteCreateRequest, RouteRenameRequest, RouteUseCase,
};

pub struct RouteController<R, O, S, I, E> {
    usecase: RouteUseCase<R, O, S, I, E>,
}

impl<R, O, S, I, E> RouteController<R, O, S, I, E>
where
    R: RouteRepository,
    O: OperationRepository,
    S: SegmentRepository,
    I: RouteInterpolationApi,
    E: ElevationApi,
{
    pub fn new(usecase: RouteUseCase<R, O, S, I, E>) -> Self {
        Self { usecase }
    }

    async fn get(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.find(id.as_ref())?))
    }

    async fn get_all(&self) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.find_all()?))
    }

    async fn get_gpx(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        let gpx_resp = self.usecase.find_gpx(id.as_ref())?;

        Ok(HttpResponse::Ok()
            .set_header(
                http::header::CONTENT_DISPOSITION,
                "attachment;filename=\"route.gpx\"",
            )
            .content_type("application/gpx+xml")
            .body(dev::Body::from_slice(gpx_resp.as_slice())))
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

    async fn patch_add(
        &self,
        path_params: web::Path<(RouteId, usize)>,
        req: web::Json<NewPointRequest>,
    ) -> Result<HttpResponse> {
        let (route_id, pos) = path_params.into_inner();
        let req = req.into_inner();
        Ok(HttpResponse::Ok().json(self.usecase.add_point(&route_id, pos, req.coord)?))
    }

    async fn patch_remove(&self, path_params: web::Path<(RouteId, usize)>) -> Result<HttpResponse> {
        let (route_id, pos) = path_params.into_inner();
        Ok(HttpResponse::Ok().json(self.usecase.remove_point(&route_id, pos)?))
    }

    async fn patch_move(
        &self,
        path_params: web::Path<(RouteId, usize)>,
        req: web::Json<NewPointRequest>,
    ) -> Result<HttpResponse> {
        let (route_id, pos) = path_params.into_inner();
        let req = req.into_inner();
        Ok(HttpResponse::Ok().json(self.usecase.move_point(&route_id, pos, req.coord)?))
    }

    async fn patch_clear(&self, route_id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.clear_route(&route_id)?))
    }

    async fn patch_undo(&self, route_id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.undo_operation(&route_id)?))
    }

    async fn patch_redo(&self, route_id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.redo_operation(&route_id)?))
    }

    async fn delete(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        self.usecase.delete(&id)?;
        Ok(HttpResponse::Ok().finish())
    }
}

pub trait BuildService<S: dev::HttpServiceFactory + 'static> {
    fn build_service(self) -> S;
}

impl<R, O, S, I, E> BuildService<Scope> for &'static Lazy<RouteController<R, O, S, I, E>>
where
    R: RouteRepository,
    O: OperationRepository,
    S: SegmentRepository,
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
            .service(web::resource("/{id}/gpx/").route(web::get().to(move |id| self.get_gpx(id))))
            .service(
                web::resource("/{id}/rename/")
                    .route(web::patch().to(move |id, req| self.patch_rename(id, req))),
            )
            .service(
                web::resource("/{id}/add/{pos}")
                    .route(web::patch().to(move |path, req| self.patch_add(path, req))),
            )
            .service(
                web::resource("/{id}/remove/{pos}")
                    .route(web::patch().to(move |path| self.patch_remove(path))),
            )
            .service(
                web::resource("/{id}/move/{pos}")
                    .route(web::patch().to(move |path, req| self.patch_move(path, req))),
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
