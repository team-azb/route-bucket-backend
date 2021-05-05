use route_bucket_backend::domain::model::operation::Operation;
use route_bucket_backend::domain::model::polyline::{Coordinate, Polyline};
use route_bucket_backend::domain::model::route::{Route, RouteEditor};
use route_bucket_backend::domain::model::types::RouteId;
use route_bucket_backend::domain::service::route::RouteService;
use route_bucket_backend::infrastructure::repository::operation::OperationRepositoryMysql;
use route_bucket_backend::infrastructure::repository::route::RouteRepositoryMysql;

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

fn main() {
    env_logger::init();

    let route_repository = RouteRepositoryMysql::new();
    let op_repository = OperationRepositoryMysql::new();
    let route_service = RouteService::new(route_repository, op_repository);

    let sample1 = Route::new(RouteId::new(), &String::from("sample1"), polyline![], 0);
    let sample2 = RouteEditor::new(
        Route::new(
            RouteId::new(),
            &"sample2".into(),
            polyline![(0.0, 100.0), (10.0, 110.0), (20.0, 120.0)],
            4,
        ),
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
            Operation::Clear {
                org_list: polyline![(0.0, 100.0), (10.0, 110.0), (20.0, 120.0)],
            },
        ],
    );

    route_service.register_route(&sample1).unwrap();
    log::info!("Route {} added!", sample1.id());
    route_service.register_editor(&sample2).unwrap();
    log::info!("Route {} added!", sample2.route().id());
}
