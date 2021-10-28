use actix_web::{dev, http, web, HttpResponse, Result};

use route_bucket_domain::model::RouteId;
use route_bucket_usecase::route::{
    NewPointRequest, RemovePointRequest, RouteCreateRequest, RouteRenameRequest, RouteUseCase,
};

use crate::AddService;

async fn get<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    id: web::Path<RouteId>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.find(id.as_ref()).await?))
}

async fn get_all<U: 'static + RouteUseCase>(usecase: web::Data<U>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.find_all().await?))
}

async fn get_gpx<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    id: web::Path<RouteId>,
) -> Result<HttpResponse> {
    let gpx_resp = usecase.find_gpx(id.as_ref()).await?;

    Ok(HttpResponse::Ok()
        .insert_header((
            http::header::CONTENT_DISPOSITION,
            format!("attachment;filename=\"{}.gpx\"", gpx_resp.name()),
        ))
        .content_type("application/gpx+xml")
        .body(dev::Body::from_slice(gpx_resp.as_slice())))
}

async fn post<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    req: web::Json<RouteCreateRequest>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(usecase.create(&req).await?))
}

async fn patch_rename<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    id: web::Path<RouteId>,
    req: web::Json<RouteRenameRequest>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.rename(&id, &req).await?))
}

async fn patch_add<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    path_params: web::Path<(RouteId, usize)>,
    req: web::Json<NewPointRequest>,
) -> Result<HttpResponse> {
    let (route_id, pos) = path_params.into_inner();
    Ok(HttpResponse::Ok().json(usecase.add_point(&route_id, pos, &req).await?))
}

async fn patch_remove<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    path_params: web::Path<(RouteId, usize)>,
    req: web::Json<RemovePointRequest>,
) -> Result<HttpResponse> {
    let (route_id, pos) = path_params.into_inner();
    Ok(HttpResponse::Ok().json(usecase.remove_point(&route_id, pos, &req).await?))
}

async fn patch_move<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    path_params: web::Path<(RouteId, usize)>,
    req: web::Json<NewPointRequest>,
) -> Result<HttpResponse> {
    let (route_id, pos) = path_params.into_inner();
    Ok(HttpResponse::Ok().json(usecase.move_point(&route_id, pos, &req).await?))
}

async fn patch_clear<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    route_id: web::Path<RouteId>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.clear_route(&route_id).await?))
}

async fn patch_undo<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    route_id: web::Path<RouteId>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.undo_operation(&route_id).await?))
}

async fn patch_redo<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    route_id: web::Path<RouteId>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.redo_operation(&route_id).await?))
}

async fn delete<U: 'static + RouteUseCase>(
    usecase: web::Data<U>,
    id: web::Path<RouteId>,
) -> Result<HttpResponse> {
    usecase.delete(&id).await?;
    Ok(HttpResponse::Ok().finish())
}

pub trait BuildRouteService: AddService {
    fn build_route_service<U: 'static + RouteUseCase>(self) -> Self {
        // TODO: /の過不足は許容する ex) "/{id}/"
        self.add_service(
            web::scope("/routes")
                .service(
                    web::resource("/")
                        .route(web::get().to(get_all::<U>))
                        .route(web::post().to(post::<U>)),
                )
                .service(
                    web::resource("/{id}")
                        .route(web::get().to(get::<U>))
                        .route(web::delete().to(delete::<U>)),
                )
                .service(web::resource("/{id}/gpx/").route(web::get().to(get_gpx::<U>)))
                .service(web::resource("/{id}/rename/").route(web::patch().to(patch_rename::<U>)))
                .service(web::resource("/{id}/add/{pos}").route(web::patch().to(patch_add::<U>)))
                .service(
                    web::resource("/{id}/remove/{pos}").route(web::patch().to(patch_remove::<U>)),
                )
                .service(web::resource("/{id}/move/{pos}").route(web::patch().to(patch_move::<U>)))
                .service(web::resource("/{id}/clear/").route(web::patch().to(patch_clear::<U>)))
                .service(web::resource("/{id}/undo/").route(web::patch().to(patch_undo::<U>)))
                .service(web::resource("/{id}/redo/").route(web::patch().to(patch_redo::<U>))),
        )
    }
}

impl<T: AddService> BuildRouteService for T {}
