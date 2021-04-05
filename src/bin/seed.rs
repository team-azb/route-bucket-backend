use diesel::associations::HasTable;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{MysqlConnection, RunQueryDsl};
use dotenv::dotenv;
use route_bucket_backend::domain::polyline::Coordinate;
use route_bucket_backend::domain::route::{Route, RouteRepository};
use route_bucket_backend::domain::types::RouteId;
use route_bucket_backend::infrastructure::dto::coordinate::CoordinateDto;
use route_bucket_backend::infrastructure::dto::route::RouteDto;
use route_bucket_backend::infrastructure::repository::route::RouteRepositoryMysql;
use std::convert::TryFrom;

// TODO: main.rsのcreate_poolと共通化
fn create_pool() -> Pool<ConnectionManager<MysqlConnection>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL NOT FOUND");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    Pool::builder()
        .max_size(4)
        .build(manager)
        .expect("Failed to create pool")
}

fn main() {
    dotenv().ok();

    env_logger::init();

    let pool = create_pool();
    let repo = RouteRepositoryMysql::new(pool);

    diesel::delete(RouteDto::table())
        .execute(&repo.get_connection().unwrap())
        .unwrap();
    diesel::delete(CoordinateDto::table())
        .execute(&repo.get_connection().unwrap())
        .unwrap();

    let sample1 = Route::new(RouteId::new(), String::from("sample1"), Vec::new());
    let sample2 = Route::new(
        RouteId::new(),
        String::from("sample2"),
        vec![
            Coordinate::try_from((0.0, 100.0)).unwrap(),
            Coordinate::try_from((10.0, 110.0)).unwrap(),
            Coordinate::try_from((20.0, 120.0)).unwrap(),
            Coordinate::try_from((30.0, 130.0)).unwrap(),
            Coordinate::try_from((40.0, 140.0)).unwrap(),
        ],
    );

    repo.register(&sample1).unwrap();
    log::info!("Route {} added!", sample1.id());
    repo.register(&sample2).unwrap();
    log::info!("Route {} added!", sample2.id());
}
