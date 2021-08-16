use actix_service::ServiceFactory;
use actix_web::body::MessageBody;
use actix_web::dev::{HttpServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::error::Error;
use actix_web::App;

pub use route::BuildRouteService;

mod route;

pub trait AddService: Sized {
    fn add_service<F>(self, factory: F) -> Self
    where
        F: HttpServiceFactory + 'static;
}

impl<T, B> AddService for App<T, B>
where
    B: MessageBody,
    T: ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<B>,
        Error = Error,
        InitError = (),
    >,
{
    fn add_service<F>(self, factory: F) -> Self
    where
        F: HttpServiceFactory + 'static,
    {
        self.service(factory)
    }
}
