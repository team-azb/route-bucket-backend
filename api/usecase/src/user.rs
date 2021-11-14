use std::convert::TryInto;

use async_trait::async_trait;
use futures::FutureExt;
use route_bucket_domain::{
    external::{CallUserAuthApi, UserAuthApi},
    repository::{CallUserRepository, Connection, Repository, UserRepository},
};
use route_bucket_utils::ApplicationResult;

pub use requests::*;
pub use responses::*;

mod requests;
mod responses;

#[async_trait]
pub trait UserUseCase {
    async fn create(&self, req: UserCreateRequest) -> ApplicationResult<UserCreateResponse>;
}

#[async_trait]
impl<T> UserUseCase for T
where
    T: CallUserRepository + CallUserAuthApi + Sync,
{
    async fn create(&self, req: UserCreateRequest) -> ApplicationResult<UserCreateResponse> {
        let (user, email, password) = req.try_into()?;

        let conn = self.user_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                // NOTE: auth api はrollbackできないので、後にやるべし
                self.user_repository().insert(&user, conn).await?;
                self.user_auth_api()
                    .create_account(&user, &email, &password)
                    .await?;

                Ok(UserCreateResponse {
                    id: user.id().clone(),
                })
            }
            .boxed()
        })
        .await
    }
}

// TODO: mod tests
