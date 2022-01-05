use actix_web::{web, HttpResponse, Result};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use route_bucket_domain::model::user::UserId;
use route_bucket_usecase::user::{
    UserCreateRequest, UserUpdateRequest, UserUseCase, UserValidateRequest,
};

use crate::AddService;

async fn get<U: 'static + UserUseCase>(
    usecase: web::Data<U>,
    id: web::Path<UserId>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.find(id.as_ref()).await?))
}

async fn post<U: 'static + UserUseCase>(
    usecase: web::Data<U>,
    req: web::Json<UserCreateRequest>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(usecase.create(req.into_inner()).await?))
}

async fn patch<U: 'static + UserUseCase>(
    usecase: web::Data<U>,
    user_id: web::Path<UserId>,
    auth: BearerAuth,
    req: web::Json<UserUpdateRequest>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(
        usecase
            .update(&user_id, auth.token(), req.into_inner())
            .await?,
    ))
}

async fn delete<U: 'static + UserUseCase>(
    usecase: web::Data<U>,
    user_id: web::Path<UserId>,
    auth: BearerAuth,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.delete(&user_id, auth.token()).await?))
}

async fn post_validate<U: 'static + UserUseCase>(
    usecase: web::Data<U>,
    req: web::Json<UserValidateRequest>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.validate(req.into_inner()).await?))
}

pub trait BuildUserService: AddService {
    fn build_user_service<U: 'static + UserUseCase>(self) -> Self {
        self.add_service(
            web::scope("/users")
                .service(web::resource("/").route(web::post().to(post::<U>)))
                .service(
                    web::resource("/{id}")
                        .route(web::get().to(get::<U>))
                        .route(web::patch().to(patch::<U>))
                        .route(web::delete().to(delete::<U>)),
                )
                .service(web::resource("/validate/").route(web::post().to(post_validate::<U>))),
        )
    }
}

impl<T: AddService> BuildUserService for T {}
