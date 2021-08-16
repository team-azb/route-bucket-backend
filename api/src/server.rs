use route_bucket_domain::external::{CallElevationApi, CallRouteInterpolationApi};
use route_bucket_domain::repository::CallRouteRepository;
use route_bucket_infrastructure::{OsrmApi, RouteRepositoryMySql, SrtmReader};

pub struct Server {
    route_repository: RouteRepositoryMySql,
    srtm_reader: SrtmReader,
    osrm_api: OsrmApi,
}

impl Server {
    pub async fn new() -> Self {
        Self {
            route_repository: RouteRepositoryMySql::new().await,
            srtm_reader: SrtmReader::new().unwrap(),
            osrm_api: OsrmApi::new(),
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
