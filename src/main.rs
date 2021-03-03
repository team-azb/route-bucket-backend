mod domain;
mod lib;

use actix_web::{HttpResponse, get, HttpServer, App, Error};
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

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    HttpServer::new(move || App::new().service(index))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}
