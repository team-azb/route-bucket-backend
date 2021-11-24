mod token;

use std::{fs::File, io::BufReader, sync::Arc};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use derivative::Derivative;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use reqwest::{Client, Response};
use route_bucket_domain::{
    external::UserAuthApi,
    model::{
        types::Email,
        user::{User, UserId},
    },
};
use route_bucket_utils::{hashmap, ApplicationError, ApplicationResult};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::RwLock;

use self::token::verify;

const CREDENTIAL_PATH: &str = "resources/credentials/firebase-adminsdk.json";
const API_SCOPE: &str = "https://www.googleapis.com/auth/identitytoolkit";
static JWT_EXP_DURATION: Lazy<Duration> = Lazy::new(|| Duration::minutes(1));
static API_TOKEN_EXP_OFFSET: Lazy<Duration> = Lazy::new(|| Duration::seconds(10));

#[derive(Clone, Debug, Deserialize, Default)]
struct FirebaseCredential {
    r#type: String,
    project_id: String,
    private_key_id: String,
    private_key: String,
    client_email: String,
    client_id: String,
    auth_uri: String,
    token_uri: String,
    auth_provider_x509_cert_url: String,
    client_x509_cert_url: String,
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
struct GoogleAccessToken {
    token: String,
    #[derivative(Default(value = "chrono::MIN_DATETIME"))]
    expires_at: DateTime<Utc>,
}

impl GoogleAccessToken {
    async fn new(credential: &FirebaseCredential) -> ApplicationResult<Self> {
        let mut token: GoogleAccessToken = Default::default();
        token.refresh(credential).await?;
        Ok(token)
    }

    fn has_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    async fn refresh(&mut self, credential: &FirebaseCredential) -> ApplicationResult<()> {
        if self.has_expired() {
            let header = Header {
                typ: Some("JWT".to_string()),
                alg: Algorithm::RS256,
                ..Default::default()
            };
            let claims = hashmap!(
                "aud" => credential.token_uri.to_string(),
                "iss" => credential.client_email.to_string(),
                "iat" => Utc::now().timestamp().to_string(),
                "exp" => (Utc::now() + *JWT_EXP_DURATION).timestamp().to_string(),
                "scope" => API_SCOPE.to_string()
            );
            let key = EncodingKey::from_rsa_pem(credential.private_key.as_bytes()).unwrap();

            let jwt = jsonwebtoken::encode(&header, &claims, &key).unwrap();

            let token_body = json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer",
                "assertion": jwt
            });

            let response = Client::new()
                .post(&claims["aud"])
                .json(&token_body)
                .send()
                .await?
                .json::<Value>()
                .await
                .unwrap();

            self.token = response
                .get("access_token")
                .ok_or_else(|| {
                    ApplicationError::AuthError(format!(
                        "Unable to find access_token in the response: {:?}",
                        response.clone()
                    ))
                })?
                .as_str()
                .unwrap()
                .to_string();

            self.expires_at = Utc::now()
                + Duration::seconds(
                    response
                        .get("expires_in")
                        .ok_or_else(|| {
                            ApplicationError::AuthError(format!(
                                "Unable to find expires_in in the response: {:?}",
                                response.clone()
                            ))
                        })?
                        .as_i64()
                        .unwrap(),
                )
                - *API_TOKEN_EXP_OFFSET;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct FirebaseAuthApi {
    credential: FirebaseCredential,
    access_token: Arc<RwLock<GoogleAccessToken>>,
}

impl FirebaseAuthApi {
    pub async fn new() -> ApplicationResult<Self> {
        let file = File::open(CREDENTIAL_PATH).unwrap();
        let reader = BufReader::new(file);

        let credential: FirebaseCredential = serde_json::from_reader(reader).unwrap();
        let access_token = Arc::new(RwLock::new(GoogleAccessToken::new(&credential).await?));

        Ok(Self {
            credential,
            access_token,
        })
    }

    async fn post_request(&self, url: String, payload: Value) -> ApplicationResult<Response> {
        if self.access_token.read().await.has_expired() {
            // Get the write-lock only when the token has expired
            let mut access_token = self.access_token.write().await;
            access_token.refresh(&self.credential).await?;
        }

        let resp = Client::new()
            .post(&url)
            .bearer_auth(&self.access_token.read().await.token)
            .json(&payload)
            .send()
            .await?;

        Ok(resp)
    }
}

#[async_trait]
impl UserAuthApi for FirebaseAuthApi {
    async fn create_account(
        &self,
        user: &User,
        email: &Email,
        password: &str,
    ) -> ApplicationResult<()> {
        let response = self
            .post_request(
                format!(
                    "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts",
                    self.credential.project_id
                ),
                json!({
                    "email": email.to_string(),
                    "password": password.to_string(),
                    "localId": user.id().to_string()
                }),
            )
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ApplicationError::AuthError(format!(
                "Failed to create account: {:?}",
                response.json::<Value>().await
            )))
        }
    }

    async fn delete_account(&self, user_id: &UserId) -> ApplicationResult<()> {
        let response = self
            .post_request(
                format!(
                    "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts:delete",
                    self.credential.project_id
                ),
                json!({
                    "localId": user_id.to_string()
                }),
            )
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ApplicationError::AuthError(format!(
                "Failed to delete account: {:?}",
                response.json::<Value>().await
            )))
        }
    }

    async fn verify_token(&self, user_id: &UserId, token: &str) -> ApplicationResult<()> {
        verify(user_id, token, &self.credential).await
    }
}
