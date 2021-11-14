use actix_web::{web, HttpResponse, Result};
use route_bucket_usecase::user::{UserCreateRequest, UserUseCase};

use crate::AddService;

async fn post<U: 'static + UserUseCase>(
    usecase: web::Data<U>,
    req: web::Json<UserCreateRequest>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(usecase.create(req.into_inner()).await?))
}

pub trait BuildUserService: AddService {
    fn build_user_service<U: 'static + UserUseCase>(self) -> Self {
        self.add_service(
            web::scope("/users").service(web::resource("/").route(web::post().to(post::<U>))),
        )
    }
}

impl<T: AddService> BuildUserService for T {}
