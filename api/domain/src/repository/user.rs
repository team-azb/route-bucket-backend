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

    async fn update(
        &self,
        user: &User,
        conn: &<Self as Repository>::Connection,
    ) -> ApplicationResult<()>;

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

mockall::mock! {
    pub UserRepository {}

    #[async_trait]
    impl Repository for UserRepository {
        type Connection = super::MockConnection;

        async fn get_connection(&self) -> ApplicationResult<super::MockConnection>;
    }

    #[async_trait]
    impl UserRepository for UserRepository {
        async fn find(&self, id: &UserId, conn: &super::MockConnection) -> ApplicationResult<User>;

        async fn insert(&self, user: &User, conn: &super::MockConnection) -> ApplicationResult<()>;

        async fn update(&self, user: &User, conn: &super::MockConnection) -> ApplicationResult<()>;

        async fn delete(&self, id: &UserId, conn: &super::MockConnection) -> ApplicationResult<()>;
    }
}
