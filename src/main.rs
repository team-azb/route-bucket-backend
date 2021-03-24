use actix_web::middleware::Logger;
use actix_web::{App, Error, HttpServer, Result};
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use once_cell::sync::Lazy;

use route_bucket_backend::controller::route::{BuildService, RouteController};
use route_bucket_backend::infrastructure::repository::route::RouteRepositoryMysql;

fn create_pool() -> Pool<ConnectionManager<MysqlConnection>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Failed to create pool")
}

// TODO: ControllerとRepositoryMysql系のstructに共通trait実装してcontrollerの初期化を↓みたいに共通化したい
// fn create_static_controller<Controller, Repository>() -> Lazy<Controller> {
//     Lazy::new(|| {
//         let pool = create_pool();
//         let route_repository = Repository::new(pool);
//         Controller::new(route_repository)
//     })
// }

type StaticRouteController = Lazy<RouteController<RouteRepositoryMysql>>;

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    env_logger::init();

    // staticじゃないと↓で怒られる
    static ROUTE_CONTROLLER: StaticRouteController = StaticRouteController::new(|| {
        let pool = create_pool();
        let route_repository = RouteRepositoryMysql::new(pool);
        RouteController::new(route_repository)
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a \"%r\" %s (%T s)"))
            .service(ROUTE_CONTROLLER.build_service())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}
