#[macro_use]
extern crate diesel;

mod domain;
mod lib;
mod infrastructure;

use actix_web::{HttpResponse, get, HttpServer, App, Error};
use dotenv::dotenv;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use nanoid::nanoid;

use crate::domain::coordinate::{Latitude, FromF64, Coordinate};
use crate::domain::route::Route;
use crate::infrastructure::dto::route::RouteDto;
use crate::infrastructure::schema;

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

    let results = schema::routes::table
        .filter(schema::routes::id.eq(&route_id))
        .limit(5)
        .load::<RouteDto>(&conn)
        .expect("failed loading");

    let response_body = format!("{:#?}", results);
    Ok(HttpResponse::Ok().body(response_body))
}

fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL NOT FOUND");

    MysqlConnection::establish(&database_url).expect("Error on db connection!")
}

#[actix_rt::main]
async fn main() -> Result<(), Error> {

    HttpServer::new(move || App::new().service(index))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}
