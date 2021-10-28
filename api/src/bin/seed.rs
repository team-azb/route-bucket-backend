use route_bucket_backend::server::Server;
use route_bucket_domain::model::{Coordinate, DrawingMode};
use route_bucket_usecase::route::RouteUseCase;

macro_rules! coord {
    ( $lat: expr, $lon: expr ) => {
        Coordinate::new($lat, $lon).unwrap()
    };
}

#[actix_web::main]
async fn main() {
    env_logger::init();

    let server = Server::new().await;

    let route_id1 = server
        .create(&String::from("sample1").into())
        .await
        .unwrap()
        .id;

    let route_id2 = server
        .create(&String::from("sample2: 皇居ラン").into())
        .await
        .unwrap()
        .id;

    server
        .add_point(
            &route_id2,
            0,
            &(DrawingMode::FollowRoad, coord!(35.68136, 139.75875)).into(),
        )
        .await
        .unwrap();
    server
        .add_point(
            &route_id2,
            1,
            &(DrawingMode::FollowRoad, coord!(35.69053, 139.75681)).into(),
        )
        .await
        .unwrap();
    server
        .add_point(
            &route_id2,
            2,
            &(DrawingMode::FollowRoad, coord!(35.69510, 139.75139)).into(),
        )
        .await
        .unwrap();
    server
        .add_point(
            &route_id2,
            3,
            &(DrawingMode::FollowRoad, coord!(35.68942, 139.74547)).into(),
        )
        .await
        .unwrap();
    server
        .add_point(
            &route_id2,
            4,
            &(DrawingMode::FollowRoad, coord!(35.68418, 139.74424)).into(),
        )
        .await
        .unwrap();
    server
        .add_point(
            &route_id2,
            5,
            &(DrawingMode::FollowRoad, coord!(35.68136, 139.75875)).into(),
        )
        .await
        .unwrap();

    log::info!("Route {} added!", route_id1);
    log::info!("Route {} added!", route_id2);
}
