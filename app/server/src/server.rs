use route_bucket_domain::external::{
    CallElevationApi, CallReservedUserIdCheckerApi, CallRouteInterpolationApi, CallUserAuthApi,
};
use route_bucket_domain::repository::{
    CallPermissionRepository, CallRouteRepository, CallUserRepository,
};
use route_bucket_infrastructure::{
    init_repositories, FirebaseAuthApi, OsrmApi, PermissionRepositoryMySql, ReservedUidsReader,
    RouteRepositoryMySql, SrtmReader, UserRepositoryMySql,
};

pub struct Server {
    route_repository: RouteRepositoryMySql,
    user_repository: UserRepositoryMySql,
    permission_repository: PermissionRepositoryMySql,
    srtm_reader: SrtmReader,
    osrm_api: OsrmApi,
    firebase_auth_api: FirebaseAuthApi,
    reserved_uids_reader: ReservedUidsReader,
}

impl Server {
    pub async fn new() -> Self {
        let (route_repository, user_repository, permission_repository) = init_repositories().await;
        Self {
            route_repository,
            user_repository,
            permission_repository,
            srtm_reader: SrtmReader::new().unwrap(),
            osrm_api: OsrmApi::new(),
            firebase_auth_api: FirebaseAuthApi::new().await.unwrap(),
            reserved_uids_reader: ReservedUidsReader::new().unwrap(),
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

impl CallPermissionRepository for Server {
    type PermissionRepository = PermissionRepositoryMySql;

    fn permission_repository(&self) -> &Self::PermissionRepository {
        &self.permission_repository
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

impl CallReservedUserIdCheckerApi for Server {
    type ReservedUserIdCheckerApi = ReservedUidsReader;

    fn reserved_user_id_checker_api(&self) -> &Self::ReservedUserIdCheckerApi {
        &self.reserved_uids_reader
    }
}
