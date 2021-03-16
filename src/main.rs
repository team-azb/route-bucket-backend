#[macro_use]
extern crate diesel;

mod domain;
mod lib;
mod infrastructure;

use actix_web::{HttpResponse, get, HttpServer, App, Error};
use actix_web::middleware::Logger;
use dotenv::dotenv;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use nanoid::nanoid;

use crate::domain::route::{Route, RouteRepository};
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::repository::route::RouteRepositoryMysql;
use crate::infrastructure::schema;
use crate::domain::types::RouteId;

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    use schema::routes::dsl::*;

    let conn = establish_connection();
    let route_id = nanoid!(11);
    let new_route = RouteDto {
        id: route_id.clone(),
        name: "sample route".to_string()
    };

    diesel::insert_into(routes)
        .values(new_route)
        .execute(&conn)
        .expect("failed inserting");

    let repository = RouteRepositoryMysql::new(conn);
    let route = repository.find(RouteId(route_id))?;
    Ok(HttpResponse::Ok().body(format!("{:#?}", route)))
}

fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL NOT FOUND");

    MysqlConnection::establish(&database_url).expect("Error on db connection!")
}

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(move || App::new().wrap(Logger::new("%a \"%r\" %s (%T s)")).service(index))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}
