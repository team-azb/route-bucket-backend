use async_trait::async_trait;
use futures::FutureExt;
use route_bucket_domain::{
    external::{
        CallReservedUserIdCheckerApi, CallUserAuthApi, ReservedUserIdCheckerApi, UserAuthApi,
    },
    model::user::{User, UserId},
    repository::{CallUserRepository, Connection, Repository, UserRepository},
};
use route_bucket_utils::ApplicationResult;
use validator::Validate;

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

    async fn validate(&self, req: UserValidateRequest) -> ApplicationResult<UserValidateResponse>;
}

#[async_trait]
impl<T> UserUseCase for T
where
    T: CallUserRepository + CallUserAuthApi + CallReservedUserIdCheckerApi + Sync,
{
    async fn find(&self, user_id: &UserId) -> ApplicationResult<User> {
        let conn = self.user_repository().get_connection().await?;

        self.user_repository().find(user_id, &conn).await
    }

    async fn create(&self, req: UserCreateRequest) -> ApplicationResult<UserCreateResponse> {
        let (user, email, password) = req.into();

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

    async fn validate(&self, req: UserValidateRequest) -> ApplicationResult<UserValidateResponse> {
        let mut resp: UserValidateResponse = Default::default();

        // Check Formats
        let validation_result = req.validate();
        if let Err(errors) = validation_result {
            let error_map = errors.into_errors();
            error_map.into_iter().for_each(|(field, _)| {
                resp.0.insert(field, ValidationErrorCode::InvalidFormat);
            });
        }

        // Check for Duplicate Uses
        if let Some(id) = req.id {
            let is_reserved = self
                .reserved_user_id_checker_api()
                .check_if_reserved(&id)
                .await?;
            if is_reserved {
                resp.0.insert("id", ValidationErrorCode::ReservedWord);
            } else {
                let conn = self.user_repository().get_connection().await?;

                let is_already_used = self.user_repository().find(&id, &conn).await.is_ok();
                if is_already_used {
                    resp.0.insert("id", ValidationErrorCode::AlreadyExists);
                };
            }
        }
        if let Some(email) = req.email {
            if self.user_auth_api().check_if_email_exists(&email).await? {
                resp.0.insert("email", ValidationErrorCode::AlreadyExists);
            };
        }

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::TryFrom, str::FromStr};

    use crate::{expect_at_repository, expect_once};
    use chrono::NaiveDate;
    use route_bucket_domain::{
        external::{MockReservedUserIdCheckerApi, MockUserAuthApi},
        model::{
            fixtures::user::{UserFixtures, UserIdFixtures},
            types::{Email, Url},
            user::{Gender, User, UserId},
        },
        repository::{MockConnection, MockUserRepository},
    };
    use route_bucket_utils::hashmap;
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

    #[fixture]
    fn unknown_email() -> Email {
        Email::try_from("unknown@email.com".to_string()).unwrap()
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

    #[rstest]
    #[tokio::test]
    async fn can_validate() {
        let req = UserValidateRequest {
            id: Some(UserId::doncic()),
            name: None,
            email: Some(unknown_email()),
            birthdate: None,
            icon_url: None,
            password: Some("short".to_string()),
        };
        let resp = UserValidateResponse(hashmap! {
            "id" => ValidationErrorCode::AlreadyExists,
            "password" => ValidationErrorCode::InvalidFormat
        });

        let mut usecase = TestUserUseCase::new();
        usecase.expect_check_if_reserved_at_uid_check_api(UserId::doncic(), false);
        usecase.expect_find_at_user_repository(UserId::doncic(), User::doncic());
        usecase.expect_check_if_email_exists_at_auth_api(unknown_email(), false);

        assert_eq!(usecase.validate(req).await, Ok(resp));
    }

    struct TestUserUseCase {
        repository: MockUserRepository,
        auth_api: MockUserAuthApi,
        uid_check_api: MockReservedUserIdCheckerApi,
    }

    // setup methods for mocking
    impl TestUserUseCase {
        fn new() -> Self {
            let mut usecase = TestUserUseCase {
                repository: MockUserRepository::new(),
                auth_api: MockUserAuthApi::new(),
                uid_check_api: MockReservedUserIdCheckerApi::new(),
            };
            expect_at_repository!(usecase.repository, get_connection, MockConnection {});

            usecase
        }

        fn expect_find_at_user_repository(&mut self, param_id: UserId, return_user: User) {
            expect_at_repository!(self.repository, find, param_id, return_user);
        }

        fn expect_insert_at_user_repository(&mut self, param_user: User) {
            expect_at_repository!(self.repository, insert, param_user, ());
        }

        fn expect_update_at_user_repository(&mut self, param_user: User) {
            expect_at_repository!(self.repository, update, param_user, ());
        }

        fn expect_delete_at_user_repository(&mut self, param_id: UserId) {
            expect_at_repository!(self.repository, delete, param_id, ());
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

        fn expect_check_if_email_exists_at_auth_api(
            &mut self,
            param_email: Email,
            result_bool: bool,
        ) {
            expect_once!(
                self.auth_api,
                check_if_email_exists,
                param_email,
                result_bool
            );
        }

        fn expect_check_if_reserved_at_uid_check_api(
            &mut self,
            param_id: UserId,
            result_bool: bool,
        ) {
            expect_once!(self.uid_check_api, check_if_reserved, param_id, result_bool);
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

    impl CallReservedUserIdCheckerApi for TestUserUseCase {
        type ReservedUserIdCheckerApi = MockReservedUserIdCheckerApi;

        fn reserved_user_id_checker_api(&self) -> &Self::ReservedUserIdCheckerApi {
            &self.uid_check_api
        }
    }
}
