use async_trait::async_trait;
use route_bucket_utils::ApplicationResult;

use crate::model::user::{User, UserId};

use super::Repository;

#[async_trait]
pub trait UserRepository: Repository {
    async fn find(
        &self,
        id: &UserId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<User>;

    async fn insert(
        &self,
        user: &User,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;

    async fn update(&self, user: &User, conn: &Self::Connection) -> ApplicationResult<()>;

    async fn delete(
        &self,
        id: &UserId,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;
}

pub trait CallUserRepository {
    type UserRepository: UserRepository;

    fn user_repository(&self) -> &Self::UserRepository;
}
