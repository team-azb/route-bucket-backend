use route_bucket_backend::domain::model::linestring::{Coordinate, LineString};
use route_bucket_backend::domain::model::operation::Operation;
use route_bucket_backend::domain::model::route::Route;
use route_bucket_backend::domain::model::types::RouteId;
use route_bucket_backend::domain::service::route::RouteService;
use route_bucket_backend::infrastructure::external::osrm::OsrmApi;
use route_bucket_backend::infrastructure::external::srtm::SrtmReader;
use route_bucket_backend::infrastructure::repository::operation::OperationRepositoryMysql;
use route_bucket_backend::infrastructure::repository::route::RouteRepositoryMysql;
use route_bucket_backend::infrastructure::repository::segment::SegmentRepositoryMysql;

macro_rules! coord {
    ( $lat: expr, $lon: expr ) => {
        Coordinate::new($lat, $lon).unwrap()
    };
}

macro_rules! linestring {
    [] => {
        LineString::from(vec![])
    };
    [ $(($lat: expr, $lon: expr)),+ $(,)?] => {
        LineString::from(vec![
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
    let segment_repository = SegmentRepositoryMysql::new();
    let osrm_api = OsrmApi::new();
    let srtm_reader = SrtmReader::new().unwrap();
    let route_service = RouteService::new(
        route_repository,
        op_repository,
        segment_repository,
        osrm_api,
        srtm_reader,
    );

    let route_id1 = RouteId::new();
    route_service
        .register_route(&Route::new(
            route_id1.clone(),
            &String::from("sample1"),
            linestring![],
            0,
        ))
        .unwrap();
    log::info!("Route {} added!", route_id1);

    let route_id2 = RouteId::new();
    route_service
        .register_route(&Route::new(
            route_id2.clone(),
            &String::from("sample2: 皇居ラン"),
            linestring![],
            0,
        ))
        .unwrap();

    let mut sample2 = route_service.find_editor(&route_id2).unwrap();

    // let sample2
    sample2
        .push_operation(Operation::Add {
            pos: 0,
            coord: coord!(35.68136, 139.75875),
        })
        .unwrap();
    route_service.update_editor(&sample2).unwrap();
    sample2
        .push_operation(Operation::Add {
            pos: 1,
            coord: coord!(35.69053, 139.75681),
        })
        .unwrap();
    route_service.update_editor(&sample2).unwrap();

    sample2
        .push_operation(Operation::Add {
            pos: 2,
            coord: coord!(35.69510, 139.75139),
        })
        .unwrap();
    route_service.update_editor(&sample2).unwrap();

    sample2
        .push_operation(Operation::Add {
            pos: 3,
            coord: coord!(35.68942, 139.74547),
        })
        .unwrap();
    route_service.update_editor(&sample2).unwrap();

    sample2
        .push_operation(Operation::Add {
            pos: 4,
            coord: coord!(35.68418, 139.74424),
        })
        .unwrap();
    route_service.update_editor(&sample2).unwrap();

    sample2
        .push_operation(Operation::Add {
            pos: 5,
            coord: coord!(35.68136, 139.75875),
        })
        .unwrap();
    route_service.update_editor(&sample2).unwrap();

    sample2
        .push_operation(Operation::Clear {
            org_list: linestring![
                (35.68136, 139.75875),
                (35.69053, 139.75681),
                (35.69510, 139.75139),
                (35.68942, 139.74547),
                (35.68418, 139.74424),
                (35.68136, 139.75875)
            ],
        })
        .unwrap();
    route_service.update_editor(&sample2).unwrap();

    sample2.undo_operation().unwrap();
    route_service.update_editor(&sample2).unwrap();

    log::info!("Route {} added!", sample2.route().id());
}
