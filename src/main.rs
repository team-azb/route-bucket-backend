#[macro_use]
extern crate diesel;

mod domain;
mod infrastructure;
mod lib;

use actix_web::middleware::Logger;
use actix_web::{get, App, Error, HttpResponse, HttpServer};
use bigdecimal::BigDecimal;
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use dotenv::dotenv;

use crate::domain::coordinate::Coordinate;
use crate::domain::route::{Route, RouteRepository};
use crate::domain::types::RouteId;
use crate::infrastructure::repository::route::RouteRepositoryMysql;
use diesel::r2d2::{ConnectionManager, Pool};

fn create_pool() -> Pool<ConnectionManager<MysqlConnection>> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Failed to create pool")
}

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a \"%r\" %s (%T s)"))
            .service(index)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}
