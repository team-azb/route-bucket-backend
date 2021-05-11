use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{App, Error, HttpServer, Result};
use once_cell::sync::Lazy;
use route_bucket_backend::controller::route::{BuildService, RouteController};
use route_bucket_backend::domain::service::route::RouteService;
use route_bucket_backend::infrastructure::repository::operation::OperationRepositoryMysql;
use route_bucket_backend::infrastructure::repository::route::RouteRepositoryMysql;
use route_bucket_backend::usecase::route::RouteUseCase;

// TODO: ControllerとRepositoryMysql系のstructに共通trait実装してcontrollerの初期化を↓みたいに共通化したい
// fn create_static_controller<Controller, Repository>() -> Lazy<Controller> {
//     Lazy::new(|| {
//         let pool = create_pool();
//         let route_repository = Repository::new(pool);
//         Controller::new(route_repository)
//     })
// }

type StaticRouteController = Lazy<RouteController<RouteRepositoryMysql, OperationRepositoryMysql>>;

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    // staticじゃないと↓で怒られる
    static ROUTE_CONTROLLER: StaticRouteController = StaticRouteController::new(|| {
        let route_repository = RouteRepositoryMysql::new();
        let operation_repository = OperationRepositoryMysql::new();
        let service = RouteService::new(route_repository, operation_repository);
        let usecase = RouteUseCase::new(service);
        RouteController::new(usecase)
    });

    HttpServer::new(move || {
        App::new()
            // TODO: swagger以外からのアクセス(or development以外の環境)ではcorsを避けたい
            .wrap(Cors::permissive())
            .wrap(Logger::new("%a \"%r\" %s (%T s)"))
            .service(ROUTE_CONTROLLER.build_service())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}
