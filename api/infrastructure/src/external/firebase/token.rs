use std::collections::HashMap;

use jsonwebtoken::{decode_header, Algorithm, DecodingKey, Validation};
use route_bucket_domain::model::user::UserId;
use route_bucket_utils::{ApplicationError, ApplicationResult};
use serde::{Deserialize, Serialize};

use super::FirebaseCredential;

const JWT_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub aud: String,
    pub iat: u64,
    pub exp: u64,
    pub iss: String,
    pub sub: String,
    pub uid: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct JWK {
    pub e: String,
    pub alg: String,
    pub kty: String,
    pub kid: String,
    pub n: String,
}

#[derive(Debug, Deserialize)]
struct KeysResponse {
    keys: Vec<JWK>,
}

impl From<KeysResponse> for HashMap<String, JWK> {
    fn from(resp: KeysResponse) -> Self {
        let mut key_map = HashMap::new();
        for key in resp.keys {
            key_map.insert(key.kid.clone(), key);
        }
        key_map
    }
}

pub(super) async fn verify(
    user_id: &UserId,
    token: &str,
    credential: &FirebaseCredential,
) -> ApplicationResult<()> {
    let kid = decode_header(token)
        .map(|header| header.kid)?
        .ok_or_else(|| {
            ApplicationError::AuthError("The decoded jwt header didn't contain kid.".into())
        })?;

    // validate: kid
    let jwks_by_kid = HashMap::from(reqwest::get(JWT_URL).await?.json::<KeysResponse>().await?);
    let jwk = jwks_by_kid.get(&kid).ok_or_else(|| {
        ApplicationError::AuthError("The decoded kid not found in jwks response".into())
    })?;

    // validate: alg, iss, aud
    let mut validation = Validation {
        iss: Some(format!(
            "https://securetoken.google.com/{}",
            credential.project_id
        )),
        ..Validation::new(Algorithm::RS256)
    };
    validation.set_audience(&[credential.project_id.clone()]);

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e);
    let decoded_token = jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation)?;

    let uid = decoded_token.claims.sub;
    (user_id.to_string() == uid).then(|| ()).ok_or_else(|| {
        ApplicationError::AuthError(format!(
            "The id of the user ({}) doesn't match the id from the token ({}).",
            user_id, uid
        ))
    })
}
