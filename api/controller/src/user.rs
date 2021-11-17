use std::{future::Future, pin::Pin};

use actix_web::{dev::Payload, web, FromRequest, HttpRequest, HttpResponse, Result};
use derive_more::Constructor;
use route_bucket_domain::model::user::UserId;
use route_bucket_usecase::user::{UserCreateRequest, UserUpdateRequest, UserUseCase};
use route_bucket_utils::ApplicationError;

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
    auth_info: UserAuthInfo,
    req: web::Json<UserUpdateRequest>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(
        usecase
            .update(&auth_info.id, &auth_info.token, req.into_inner())
            .await?,
    ))
}

async fn delete<U: 'static + UserUseCase>(
    usecase: web::Data<U>,
    auth_info: UserAuthInfo,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(usecase.delete(&auth_info.id, &auth_info.token).await?))
}

#[derive(Clone, Debug, Constructor)]
struct UserAuthInfo {
    id: UserId,
    token: String,
}

impl FromRequest for UserAuthInfo {
    type Config = ();
    type Error = ApplicationError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(auth_val) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_val.to_str() {
                if auth_str.starts_with("bearer") || auth_str.starts_with("Bearer") {
                    let id = req.match_info().load().unwrap();
                    let token = auth_str[6..auth_str.len()].trim().to_string();
                    return Box::pin(async move { Ok(UserAuthInfo::new(id, token)) });
                }
            }
        }

        Box::pin(async move { Err(ApplicationError::AuthError("Token not found!".into())) })
    }

    fn extract(req: &HttpRequest) -> Self::Future {
        Self::from_request(req, &mut actix_web::dev::Payload::None)
    }

    fn configure<F>(f: F) -> Self::Config
    where
        F: FnOnce(Self::Config) -> Self::Config,
    {
        f(Self::Config::default())
    }
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
                ),
        )
    }
}

impl<T: AddService> BuildUserService for T {}
