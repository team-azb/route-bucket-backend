use route_bucket_domain::external::{CallElevationApi, CallRouteInterpolationApi, CallUserAuthApi};
use route_bucket_domain::repository::{CallRouteRepository, CallUserRepository};
use route_bucket_infrastructure::{
    init_repositories, FirebaseAuthApi, OsrmApi, RouteRepositoryMySql, SrtmReader,
    UserRepositoryMySql,
};

pub struct Server {
    route_repository: RouteRepositoryMySql,
    user_repository: UserRepositoryMySql,
    srtm_reader: SrtmReader,
    osrm_api: OsrmApi,
    firebase_auth_api: FirebaseAuthApi,
}

impl Server {
    pub async fn new() -> Self {
        let (route_repository, user_repository) = init_repositories().await;
        Self {
            route_repository,
            user_repository,
            srtm_reader: SrtmReader::new().unwrap(),
            osrm_api: OsrmApi::new(),
            firebase_auth_api: FirebaseAuthApi::new().await.unwrap(),
        }
    }
}

// TODO: この辺のboiler plateたちをmacroでどうにかする
impl CallRouteRepository for Server {
    type RouteRepository = RouteRepositoryMySql;

    fn route_repository(&self) -> &Self::RouteRepository {
        &self.route_repository
    }
}

impl CallUserRepository for Server {
    type UserRepository = UserRepositoryMySql;

    fn user_repository(&self) -> &Self::UserRepository {
        &self.user_repository
    }
}

impl CallElevationApi for Server {
    type ElevationApi = SrtmReader;

    fn elevation_api(&self) -> &Self::ElevationApi {
        &self.srtm_reader
    }
}

impl CallRouteInterpolationApi for Server {
    type RouteInterpolationApi = OsrmApi;

    fn route_interpolation_api(&self) -> &Self::RouteInterpolationApi {
        &self.osrm_api
    }
}

impl CallUserAuthApi for Server {
    type UserAuthApi = FirebaseAuthApi;

    fn user_auth_api(&self) -> &Self::UserAuthApi {
        &self.firebase_auth_api
    }
}
