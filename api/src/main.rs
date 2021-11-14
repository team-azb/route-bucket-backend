use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, Error, HttpServer, Result};

use route_bucket_backend::server::Server;
use route_bucket_controller::{BuildRouteService, BuildUserService};

#[actix_web::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let server = web::Data::new(Server::new().await);

    HttpServer::new(move || {
        App::new()
            // TODO: swagger以外からのアクセス(or development以外の環境)ではcorsを避けたい
            .wrap(Cors::permissive())
            .wrap(Logger::new("%a \"%r\" %s (%T s)"))
            .app_data(server.clone())
            .build_route_service::<Server>()
            .build_user_service::<Server>()
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}
