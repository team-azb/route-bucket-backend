mod domain;
mod lib;

use actix_web::{HttpResponse, get, HttpServer, App, Error};
use dotenv::dotenv;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;

use crate::domain::coordinate::{Latitude, FromF64, Coordinate};
use crate::domain::route::Route;

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
    let tokyo_station_coord = Coordinate::create(35.680, 139.767)?;
    let disney_land_coord = Coordinate::create(35.632, 139.880)?;

    let mut route = Route::new("From Tokyo Station to Disney Land");

    route.add_point(tokyo_station_coord);
    route.add_point(disney_land_coord);

    let response_body = format!("{:?}", route);
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
