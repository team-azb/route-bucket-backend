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
use nanoid::nanoid;

use crate::domain::coordinate::Coordinate;
use crate::domain::route::{Route, RouteRepository};
use crate::domain::types::RouteId;
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::repository::route::RouteRepositoryMysql;
use crate::infrastructure::schema;

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    use schema::routes::dsl::*;

    let conn = establish_connection();
    let repository = RouteRepositoryMysql::new(conn);

    let route = Route::new(
        RouteId::new(),
        "sample route".to_string(),
        vec![
            Coordinate::new(BigDecimal::from(35.0), BigDecimal::from(130.0))?,
            Coordinate::new(BigDecimal::from(45.0), BigDecimal::from(140.0))?,
        ],
    );

    repository.register(&route)?;

    let route = repository.find(&route.id())?;
    Ok(HttpResponse::Ok().body(format!("{:#?}", route)))
}

fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");

    MysqlConnection::establish(&database_url).expect("Error on db connection!")
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
