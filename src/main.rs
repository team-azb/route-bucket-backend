use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{App, Error, HttpServer, Result};
use route_bucket_backend::controller::route::{BuildService, RouteController};
use route_bucket_backend::infrastructure::external::osrm::OsrmApi;
use route_bucket_backend::infrastructure::external::srtm::SrtmReader;
use route_bucket_backend::infrastructure::repository::route::RouteRepositoryMySql;
use route_bucket_backend::usecase::route::RouteUseCase;
use tokio::sync::OnceCell;

// TODO: ControllerとRepositoryMysql系のstructに共通trait実装してcontrollerの初期化を↓みたいに共通化したい
// fn create_static_controller<Controller, Repository>() -> Lazy<Controller> {
//     Lazy::new(|| {
//         let pool = create_pool();
//         let route_repository = Repository::new(pool);
//         Controller::new(route_repository)
//     })
// }

type StaticRouteController = OnceCell<RouteController<RouteRepositoryMySql, OsrmApi, SrtmReader>>;

#[actix_web::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    // staticじゃないと↓で怒られる
    static ROUTE_CONTROLLER: StaticRouteController = OnceCell::const_new();
    ROUTE_CONTROLLER
        .get_or_init(|| async {
            let route_repository = RouteRepositoryMySql::new().await;
            let osrm_api = OsrmApi::new();
            let srtm_reader = SrtmReader::new().unwrap();
            let usecase = RouteUseCase::new(route_repository, osrm_api, srtm_reader);
            RouteController::new(usecase)
        })
        .await;

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
