use actix_web::{dev, http, web, HttpResponse, Result, Scope};
use tokio::sync::OnceCell;

use route_bucket_domain::model::RouteId;
use route_bucket_domain::repository::{ElevationApi, RouteInterpolationApi, RouteRepository};
use route_bucket_usecase::{NewPointRequest, RouteCreateRequest, RouteRenameRequest, RouteUseCase};

pub struct RouteController<R, I, E> {
    usecase: RouteUseCase<R, I, E>,
}

impl<R, I, E> RouteController<R, I, E>
where
    R: RouteRepository,
    I: RouteInterpolationApi,
    E: ElevationApi,
{
    pub fn new(usecase: RouteUseCase<R, I, E>) -> Self {
        Self { usecase }
    }

    async fn get(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.find(id.as_ref()).await?))
    }

    async fn get_all(&self) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.find_all().await?))
    }

    async fn get_gpx(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        let gpx_resp = self.usecase.find_gpx(id.as_ref()).await?;

        Ok(HttpResponse::Ok()
            .insert_header((
                http::header::CONTENT_DISPOSITION,
                "attachment;filename=\"route.gpx\"",
            ))
            .content_type("application/gpx+xml")
            .body(dev::Body::from_slice(gpx_resp.as_slice())))
    }

    async fn post(&self, req: web::Json<RouteCreateRequest>) -> Result<HttpResponse> {
        Ok(HttpResponse::Created().json(self.usecase.create(&req).await?))
    }

    async fn patch_rename(
        &self,
        id: web::Path<RouteId>,
        req: web::Json<RouteRenameRequest>,
    ) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.rename(&id, &req).await?))
    }

    async fn patch_add(
        &self,
        path_params: web::Path<(RouteId, usize)>,
        req: web::Json<NewPointRequest>,
    ) -> Result<HttpResponse> {
        let (route_id, pos) = path_params.into_inner();
        Ok(HttpResponse::Ok().json(self.usecase.add_point(&route_id, pos, &req).await?))
    }

    async fn patch_remove(&self, path_params: web::Path<(RouteId, usize)>) -> Result<HttpResponse> {
        let (route_id, pos) = path_params.into_inner();
        Ok(HttpResponse::Ok().json(self.usecase.remove_point(&route_id, pos).await?))
    }

    async fn patch_move(
        &self,
        path_params: web::Path<(RouteId, usize)>,
        req: web::Json<NewPointRequest>,
    ) -> Result<HttpResponse> {
        let (route_id, pos) = path_params.into_inner();
        Ok(HttpResponse::Ok().json(self.usecase.move_point(&route_id, pos, &req).await?))
    }

    async fn patch_clear(&self, route_id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.clear_route(&route_id).await?))
    }

    async fn patch_undo(&self, route_id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.undo_operation(&route_id).await?))
    }

    async fn patch_redo(&self, route_id: web::Path<RouteId>) -> Result<HttpResponse> {
        Ok(HttpResponse::Ok().json(self.usecase.redo_operation(&route_id).await?))
    }

    async fn delete(&self, id: web::Path<RouteId>) -> Result<HttpResponse> {
        self.usecase.delete(&id).await?;
        Ok(HttpResponse::Ok().finish())
    }
}

pub trait BuildService<S: dev::HttpServiceFactory + 'static> {
    fn build_service(self) -> S;
}

impl<R, I, E> BuildService<Scope> for &'static OnceCell<RouteController<R, I, E>>
where
    R: RouteRepository,
    I: RouteInterpolationApi,
    E: ElevationApi,
{
    fn build_service(self) -> Scope {
        let controller = self.get().unwrap();
        // TODO: /の過不足は許容する ex) "/{id}/"
        web::scope("/routes")
            .service(
                web::resource("/")
                    .route(web::get().to(move || controller.get_all()))
                    .route(web::post().to(move |req| controller.post(req))),
            )
            .service(
                web::resource("/{id}")
                    .route(web::get().to(move |id| controller.get(id)))
                    .route(web::delete().to(move |id| controller.delete(id))),
            )
            .service(
                web::resource("/{id}/gpx/").route(web::get().to(move |id| controller.get_gpx(id))),
            )
            .service(
                web::resource("/{id}/rename/")
                    .route(web::patch().to(move |id, req| controller.patch_rename(id, req))),
            )
            .service(
                web::resource("/{id}/add/{pos}")
                    .route(web::patch().to(move |path, req| controller.patch_add(path, req))),
            )
            .service(
                web::resource("/{id}/remove/{pos}")
                    .route(web::patch().to(move |path| controller.patch_remove(path))),
            )
            .service(
                web::resource("/{id}/move/{pos}")
                    .route(web::patch().to(move |path, req| controller.patch_move(path, req))),
            )
            .service(
                web::resource("/{id}/clear/")
                    .route(web::patch().to(move |id| controller.patch_clear(id))),
            )
            .service(
                web::resource("/{id}/undo/")
                    .route(web::patch().to(move |id| controller.patch_undo(id))),
            )
            .service(
                web::resource("/{id}/redo/")
                    .route(web::patch().to(move |id| controller.patch_redo(id))),
            )
    }
}
