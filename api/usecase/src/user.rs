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

                Ok(user.id().clone().into())
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
            password_confirmation: "LukaMagic#77".to_string(),
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
            password_confirmation: "LukaMagic#77".to_string(),
        }
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

        fn expect_insert_at_user_repository(&mut self, user: User) {
            expect_at_repository!(self, insert, user, ());
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
