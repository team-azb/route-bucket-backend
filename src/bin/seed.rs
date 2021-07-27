use route_bucket_domain::model::Coordinate;
use route_bucket_infrastructure::{OsrmApi, RouteRepositoryMySql, SrtmReader};
use route_bucket_usecase::RouteUseCase;

macro_rules! coord {
    ( $lat: expr, $lon: expr ) => {
        Coordinate::new($lat, $lon).unwrap()
    };
}

#[actix_web::main]
async fn main() {
    env_logger::init();

    let route_repository = RouteRepositoryMySql::new().await;
    let osrm_api = OsrmApi::new();
    let srtm_reader = SrtmReader::new().unwrap();
    let route_usecase = RouteUseCase::new(route_repository, osrm_api, srtm_reader);

    let route_id1 = route_usecase
        .create(&String::from("sample1").into())
        .await
        .unwrap()
        .id;

    let route_id2 = route_usecase
        .create(&String::from("sample2: 皇居ラン").into())
        .await
        .unwrap()
        .id;

    route_usecase
        .add_point(&route_id2, 0, &coord!(35.68136, 139.75875).into())
        .await
        .unwrap();
    route_usecase
        .add_point(&route_id2, 1, &coord!(35.69053, 139.75681).into())
        .await
        .unwrap();
    route_usecase
        .add_point(&route_id2, 2, &coord!(35.69510, 139.75139).into())
        .await
        .unwrap();
    route_usecase
        .add_point(&route_id2, 3, &coord!(35.68942, 139.74547).into())
        .await
        .unwrap();
    route_usecase
        .add_point(&route_id2, 4, &coord!(35.68418, 139.74424).into())
        .await
        .unwrap();
    route_usecase
        .add_point(&route_id2, 5, &coord!(35.68136, 139.75875).into())
        .await
        .unwrap();

    route_usecase.clear_route(&route_id2).await.unwrap();
    route_usecase.undo_operation(&route_id2).await.unwrap();

    log::info!("Route {} added!", route_id1);
    log::info!("Route {} added!", route_id2);
}
