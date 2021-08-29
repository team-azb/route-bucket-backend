use async_trait::async_trait;

use route_bucket_domain::model::{User, UserId};
use route_bucket_domain::repository::{CallUserRepository, Repository, UserRepository};
use route_bucket_utils::ApplicationResult;

#[async_trait]
pub trait UserUseCase {
    async fn find(&self, user_id: &UserId) -> ApplicationResult<UserGetResponse>;

    async fn create(&self, req: &UserCreateRequest) -> ApplicationResult<()>;
}

#[async_trait]
impl<T> UserUseCase for T
where
    T: CallUserRepository + Sync,
{
    async fn find(&self, user_id: &UserId) -> ApplicationResult<UserGetResponse> {
        let conn = self.user_repository().get_connection().await?;
        self.user_repository().find(user_id, &conn).await
    }

    async fn create(&self, req: &UserCreateRequest) -> ApplicationResult<()> {
        let conn = self.user_repository().get_connection().await?;
        self.user_repository().insert(req, &conn).await
    }
}

// TODO: request, response用のファイルを作る
pub type UserCreateRequest = User;

pub type UserGetResponse = User;

pub type UserCreateResponse = User;
