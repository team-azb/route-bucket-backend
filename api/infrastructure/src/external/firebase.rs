use std::{fs::File, io::BufReader};

use async_trait::async_trait;
use chrono::{DateTime, Duration, TimeZone, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use reqwest::Client;
use route_bucket_domain::{
    external::UserAuthApi,
    model::{
        types::Email,
        user::{User, UserAuthInfo},
    },
};
use route_bucket_utils::{hashmap, ApplicationError, ApplicationResult};
use serde::Deserialize;
use serde_json::{json, Value};

const CREDENTIAL_PATH: &str = "resources/credentials/firebase-adminsdk.json";
const API_SCOPE: &str = "https://www.googleapis.com/auth/identitytoolkit";
const JWT_EXP_DURATION: Lazy<Duration> = Lazy::new(|| Duration::minutes(1));

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

#[derive(Clone, Debug)]
pub struct AccessToken {
    token: String,
    expires_at: DateTime<Utc>,
}

impl Default for AccessToken {
    fn default() -> Self {
        Self {
            expires_at: Utc.timestamp(0, 0),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FirebaseAuthApi {
    credential: FirebaseCredential,
    access_token: String,
}

impl FirebaseAuthApi {
    pub async fn new() -> ApplicationResult<Self> {
        let file = File::open(CREDENTIAL_PATH).unwrap();
        let reader = BufReader::new(file);
        let credential: FirebaseCredential = serde_json::from_reader(reader).unwrap();

        let mut api = Self {
            credential,
            ..Default::default()
        };

        api.update_access_token().await?;

        Ok(api)
    }

    pub async fn update_access_token(&mut self) -> ApplicationResult<()> {
        let header = Header {
            typ: Some("JWT".to_string()),
            alg: Algorithm::RS256,
            ..Default::default()
        };
        let claims = hashmap!(
            "aud" => self.credential.token_uri.to_string(),
            "iss" => self.credential.client_email.to_string(),
            "iat" => Utc::now().timestamp().to_string(),
            "exp" => (Utc::now() + *JWT_EXP_DURATION).timestamp().to_string(),
            "scope" => API_SCOPE.to_string()
        );
        let key = EncodingKey::from_rsa_pem(self.credential.private_key.as_bytes()).unwrap();

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

        self.access_token = response
            .get("access_token")
            .ok_or_else(|| {
                ApplicationError::ExternalError(format!(
                    "Unable to find access_token in the response: {:?}",
                    response.clone()
                ))
            })?
            .as_str()
            .unwrap()
            .to_string();

        Ok(())
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
        let payload = json!({
            "displayName": user.name().to_string(),
            "email": email.to_string(),
            "password": password.to_string(),
            "localId": user.id().to_string()
        });
        let response = Client::new()
            .post(format!(
                "https://identitytoolkit.googleapis.com/v1/projects/{}/accounts",
                self.credential.project_id
            ))
            .bearer_auth(&self.access_token)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ApplicationError::ExternalError(format!(
                "Failed to create account: {:?}",
                response.json::<Value>().await
            )))
        }
    }

    #[allow(unused_variables)]
    async fn verify_token(&self, auth_info: &UserAuthInfo) -> ApplicationResult<()> {
        todo!()
    }
}
