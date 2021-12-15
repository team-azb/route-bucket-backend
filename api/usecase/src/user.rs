use std::convert::TryInto;

use async_trait::async_trait;
use futures::FutureExt;
use route_bucket_domain::{
    external::{CallUserAuthApi, UserAuthApi},
    model::user::{User, UserId},
    repository::{CallUserRepository, Connection, Repository, UserRepository},
};
use route_bucket_utils::ApplicationResult;

pub use requests::*;
pub use responses::*;

mod requests;
mod responses;

#[async_trait]
pub trait UserUseCase {
    async fn find(&self, user_id: &UserId) -> ApplicationResult<User>;

    async fn create(&self, req: UserCreateRequest) -> ApplicationResult<UserCreateResponse>;

    async fn update(
        &self,
        user_id: &UserId,
        token: &str,
        req: UserUpdateRequest,
    ) -> ApplicationResult<User>;

    async fn delete(&self, user_id: &UserId, token: &str) -> ApplicationResult<()>;
}

#[async_trait]
impl<T> UserUseCase for T
where
    T: CallUserRepository + CallUserAuthApi + Sync,
{
    async fn find(&self, user_id: &UserId) -> ApplicationResult<User> {
        let conn = self.user_repository().get_connection().await?;

        self.user_repository().find(user_id, &conn).await
    }

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

                Ok(user.id().clone().into())
            }
            .boxed()
        })
        .await
    }

    async fn update(
        &self,
        user_id: &UserId,
        token: &str,
        req: UserUpdateRequest,
    ) -> ApplicationResult<User> {
        let conn = self.user_repository().get_connection().await?;

        conn.transaction(|conn| {
            async move {
                self.user_auth_api().authorize(user_id, token).await?;

                let mut user = self.user_repository().find(user_id, conn).await?;

                req.name.map(|name| user.set_name(name));
                req.gender.map(|gender| user.set_gender(gender));
                req.birthdate
                    .map(|birthdate| user.set_birthdate(Some(birthdate)));
                req.icon_url
                    .map(|icon_url| user.set_icon_url(Some(icon_url)));

                self.user_repository().update(&user, conn).await?;

                Ok(user)
            }
            .boxed()
        })
        .await
    }

    async fn delete(&self, user_id: &UserId, token: &str) -> ApplicationResult<()> {
        let conn = self.user_repository().get_connection().await?;
        conn.transaction(|conn| {
            async move {
                self.user_auth_api().authorize(user_id, token).await?;

                self.user_repository().delete(user_id, conn).await?;
                self.user_auth_api().delete_account(user_id).await
            }
            .boxed()
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::TryFrom, str::FromStr};

    use crate::{expect_at_repository, expect_once};
    use chrono::NaiveDate;
    use route_bucket_domain::{
        external::MockUserAuthApi,
        model::{
            fixtures::user::{UserFixtures, UserIdFixtures},
            types::{Email, Url},
            user::{Gender, User, UserId},
        },
        repository::{MockConnection, MockUserRepository},
    };
    use rstest::{fixture, rstest};

    use super::*;

    #[fixture]
    fn doncic_create_request() -> UserCreateRequest {
        UserCreateRequest {
            id: UserId::doncic(),
            name: "Luka Doncic".to_string(),
            email: Email::try_from("luka77@mavs.com".to_string()).unwrap(),
            gender: Gender::Male,
            birthdate: NaiveDate::from_str("1999-02-28").ok(),
            icon_url: Url::try_from("https://on.nba.com/30qMUEI".to_string()).ok(),
            password: "LukaMagic#77".to_string(),
        }
    }

    #[fixture]
    fn porzingis_create_request() -> UserCreateRequest {
        UserCreateRequest {
            id: UserId::porzingis(),
            name: "Kristaps Porzingis".to_string(),
            email: Email::try_from("porzingis6@mavs.com".to_string()).unwrap(),
            gender: Gender::Others,
            birthdate: None,
            icon_url: None,
            password: "LukaMagic#77".to_string(),
        }
    }

    #[fixture]
    fn update_to_doncic_request() -> UserUpdateRequest {
        UserUpdateRequest {
            name: Some("Luka Doncic".to_string()),
            gender: Some(Gender::Male),
            birthdate: NaiveDate::from_str("1999-02-28").ok(),
            icon_url: Url::try_from("https://on.nba.com/30qMUEI".to_string()).ok(),
        }
    }

    #[fixture]
    fn doncic_token() -> String {
        String::from("token.for.doncic")
    }

    #[fixture]
    fn porzingis_token() -> String {
        String::from("token.for.porzingis")
    }

    #[rstest]
    #[tokio::test]
    async fn can_find() {
        let mut usecase = TestUserUseCase::new();
        usecase.expect_find_at_user_repository(UserId::doncic(), User::doncic());

        assert_eq!(usecase.find(&UserId::doncic()).await, Ok(User::doncic()));
    }

    #[rstest]
    #[case::minimum_profile(porzingis_create_request(), User::porzingis())]
    #[case::full_profile(doncic_create_request(), User::doncic())]
    #[tokio::test]
    async fn can_create(#[case] req: UserCreateRequest, #[case] user: User) {
        let mut usecase = TestUserUseCase::new();
        usecase.expect_create_account_at_auth_api(
            user.clone(),
            req.email.clone(),
            req.password.clone(),
        );
        usecase.expect_insert_at_user_repository(user.clone());

        assert_eq!(usecase.create(req).await, Ok(user.id().clone().into()));
    }

    #[rstest]
    #[tokio::test]
    async fn can_update() {
        let mut usecase = TestUserUseCase::new();
        usecase.expect_authorize_at_auth_api(UserId::porzingis(), porzingis_token());
        usecase.expect_find_at_user_repository(UserId::porzingis(), User::porzingis());
        usecase.expect_update_at_user_repository(User::porzingis_pretending_like_doncic());

        assert_eq!(
            usecase
                .update(
                    &UserId::porzingis(),
                    &porzingis_token(),
                    update_to_doncic_request()
                )
                .await,
            Ok(User::porzingis_pretending_like_doncic())
        );
    }

    #[rstest]
    #[tokio::test]
    async fn can_delete() {
        let mut usecase = TestUserUseCase::new();
        usecase.expect_authorize_at_auth_api(UserId::doncic(), doncic_token());
        usecase.expect_delete_account_at_auth_api(UserId::doncic());
        usecase.expect_delete_at_user_repository(UserId::doncic());

        assert_eq!(
            usecase.delete(&UserId::doncic(), &doncic_token()).await,
            Ok(())
        );
    }

    struct TestUserUseCase {
        repository: MockUserRepository,
        auth_api: MockUserAuthApi,
    }

    // setup methods for mocking
    impl TestUserUseCase {
        fn new() -> Self {
            let mut usecase = TestUserUseCase {
                repository: MockUserRepository::new(),
                auth_api: MockUserAuthApi::new(),
            };
            expect_at_repository!(usecase, get_connection, MockConnection {});

            usecase
        }

        fn expect_find_at_user_repository(&mut self, param_id: UserId, return_user: User) {
            expect_at_repository!(self, find, param_id, return_user);
        }

        fn expect_insert_at_user_repository(&mut self, param_user: User) {
            expect_at_repository!(self, insert, param_user, ());
        }

        fn expect_update_at_user_repository(&mut self, param_user: User) {
            expect_at_repository!(self, update, param_user, ());
        }

        fn expect_delete_at_user_repository(&mut self, param_id: UserId) {
            expect_at_repository!(self, delete, param_id, ());
        }

        fn expect_create_account_at_auth_api(
            &mut self,
            param_user: User,
            param_email: Email,
            param_password: String,
        ) {
            expect_once!(
                self.auth_api,
                create_account,
                param_user,
                param_email,
                param_password,
                ()
            );
        }

        fn expect_delete_account_at_auth_api(&mut self, param_id: UserId) {
            expect_once!(self.auth_api, delete_account, param_id, ());
        }

        fn expect_authorize_at_auth_api(&mut self, param_id: UserId, param_token: String) {
            expect_once!(self.auth_api, authorize, param_id, param_token, ());
        }
    }

    // impls to enable trait RouteUseCase
    impl CallUserRepository for TestUserUseCase {
        type UserRepository = MockUserRepository;

        fn user_repository(&self) -> &Self::UserRepository {
            &self.repository
        }
    }

    impl CallUserAuthApi for TestUserUseCase {
        type UserAuthApi = MockUserAuthApi;

        fn user_auth_api(&self) -> &Self::UserAuthApi {
            &self.auth_api
        }
    }
}
