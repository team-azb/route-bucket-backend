use diesel::associations::HasTable;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{MysqlConnection, RunQueryDsl};
use dotenv::dotenv;
use route_bucket_backend::domain::operation_history::{Operation, OperationHistory};
use route_bucket_backend::domain::polyline::{Coordinate, Polyline};
use route_bucket_backend::domain::route::{Route, RouteRepository};
use route_bucket_backend::domain::types::RouteId;
use route_bucket_backend::infrastructure::dto::operation::OperationDto;
use route_bucket_backend::infrastructure::dto::route::RouteDto;
use route_bucket_backend::infrastructure::repository::route::RouteRepositoryMysql;
use std::convert::TryFrom;

macro_rules! coord {
    ( $lat: expr, $lon: expr ) => {
        Coordinate::new($lat, $lon).unwrap()
    };
}

macro_rules! polyline {
    [] => {
        Polyline::from_vec(vec![])
    };
    [ $(($lat: expr, $lon: expr)),+ $(,)?] => {
        Polyline::from_vec(vec![
            $(
                coord!($lat, $lon),
            )+
        ])
    };
}

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
    diesel::delete(OperationDto::table())
        .execute(&repo.get_connection().unwrap())
        .unwrap();

    let sample1 = Route::new(
        RouteId::new(),
        &String::from("sample1"),
        polyline![],
        OperationHistory::new(vec![], 0),
    );
    let sample2 = Route::new(
        RouteId::new(),
        &String::from("sample2"),
        polyline![(0.0, 100.0), (10.0, 110.0), (20.0, 120.0),],
        OperationHistory::new(
            vec![
                Operation::InitWithList {
                    list: polyline![(10.0, 110.0), (50.0, 150.0)],
                },
                Operation::Add {
                    pos: 0,
                    coord: coord!(0.0, 100.0),
                },
                Operation::Add {
                    pos: 2,
                    coord: coord!(20.0, 120.0),
                },
                Operation::Remove {
                    pos: 3,
                    coord: coord!(50.0, 150.0),
                },
            ],
            4,
        ),
    );

    repo.register(&sample1).unwrap();
    log::info!("Route {} added!", sample1.id());
    repo.register(&sample2).unwrap();
    log::info!("Route {} added!", sample2.id());
}
