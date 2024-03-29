use async_trait::async_trait;
use itertools::Itertools;

use route_bucket_utils::{ApplicationError, ApplicationResult};

use crate::model::{
    route::{Coordinate, DrawingMode, Elevation, Route, Segment},
    types::Email,
    user::{User, UserId},
};

#[cfg_attr(feature = "mocking", mockall::automock)]
#[async_trait]
pub trait RouteInterpolationApi: Send + Sync {
    async fn correct_coordinate(
        &self,
        coord: &Coordinate,
        mode: DrawingMode,
    ) -> ApplicationResult<Coordinate>;

    async fn interpolate(&self, seg: &mut Segment) -> ApplicationResult<()>;

    async fn interpolate_empty_segments(&self, route: &mut Route) -> ApplicationResult<()> {
        let seg_future_iter = route
            .iter_seg_mut()
            .filter(|seg| seg.is_empty())
            .map(|seg| async move { self.interpolate(seg).await });

        futures::future::join_all(seg_future_iter)
            .await
            .into_iter()
            .try_collect()
    }
}

pub trait CallRouteInterpolationApi {
    type RouteInterpolationApi: RouteInterpolationApi;

    fn route_interpolation_api(&self) -> &Self::RouteInterpolationApi;
}

#[cfg_attr(feature = "mocking", mockall::automock)]
pub trait ElevationApi: Send + Sync {
    fn get_elevation(&self, coord: &Coordinate) -> ApplicationResult<Option<Elevation>>;

    fn attach_elevations(&self, route: &mut Route) -> ApplicationResult<()> {
        route.iter_seg_mut().try_for_each(|seg| {
            seg.iter_mut()
                .filter(|coord| coord.elevation().is_none())
                .try_for_each(|coord| coord.set_elevation(self.get_elevation(coord)?))
        })
    }
}

pub trait CallElevationApi {
    type ElevationApi: ElevationApi;

    fn elevation_api(&self) -> &Self::ElevationApi;
}

#[cfg_attr(feature = "mocking", mockall::automock)]
#[async_trait]
pub trait UserAuthApi: Send + Sync {
    async fn create_account(
        &self,
        user: &User,
        email: &Email,
        password: &str,
    ) -> ApplicationResult<()>;

    async fn delete_account(&self, user_id: &UserId) -> ApplicationResult<()>;

    async fn authenticate(&self, token: &str) -> ApplicationResult<UserId>;

    async fn authorize(&self, user_id: &UserId, token: &str) -> ApplicationResult<()> {
        let id_from_token = self.authenticate(token).await?;
        (*user_id == id_from_token).then(|| ()).ok_or_else(|| {
            ApplicationError::AuthorizationError(format!(
                "The id of the user ({}) doesn't match the id from the token ({}).",
                user_id, id_from_token
            ))
        })
    }

    async fn check_if_email_exists(&self, email: &Email) -> ApplicationResult<bool>;
}

pub trait CallUserAuthApi {
    type UserAuthApi: UserAuthApi;

    fn user_auth_api(&self) -> &Self::UserAuthApi;
}

#[cfg_attr(feature = "mocking", mockall::automock)]
#[async_trait]
pub trait ReservedUserIdCheckerApi: Send + Sync {
    async fn check_if_reserved(&self, id: &UserId) -> ApplicationResult<bool>;
}

pub trait CallReservedUserIdCheckerApi {
    type ReservedUserIdCheckerApi: ReservedUserIdCheckerApi;

    fn reserved_user_id_checker_api(&self) -> &Self::ReservedUserIdCheckerApi;
}
