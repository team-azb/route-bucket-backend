use std::collections::HashMap;
use std::fs::File;
use std::future::Future;
use std::io::BufReader;
use std::pin::Pin;

use actix_web::dev::Payload;
use actix_web::{web, FromRequest, HttpRequest, HttpResponse, Result};
use jsonwebtoken::{decode_header, Algorithm, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};

use route_bucket_domain::model::UserId;
use route_bucket_usecase::user::{UserCreateRequest, UserUseCase};
use route_bucket_utils::ApplicationError;

use crate::AddService;

#[derive(Clone, Debug, Deserialize)]
pub struct FirebaseConfig {
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
}

impl FirebaseConfig {
    pub fn new() -> FirebaseConfig {
        let file = File::open("credentials/firebase-adminsdk.json").unwrap();
        let reader = BufReader::new(file);
        let config: FirebaseConfig = serde_json::from_reader(reader).unwrap();
        config
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: String,
    pub iat: u64,
    pub exp: u64,
    pub iss: String,
    pub sub: String,
    pub uid: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct JWK {
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

pub async fn verify(
    token: &str,
    firebase_config: &FirebaseConfig,
) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let kid = match decode_header(token).map(|header| header.kid) {
        Ok(Some(k)) => k,
        Ok(None) => {
            return Err(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::__Nonexhaustive,
            ))
        }
        Err(err) => return Err(err),
    };

    // validate: kid
    let jwks = get_firebase_jwks().await.unwrap();
    let jwk = jwks.get(&kid).unwrap();

    // validate: alg, iss, aud
    let mut validation = Validation {
        iss: Some(format!(
            "https://securetoken.google.com/{}",
            firebase_config.project_id
        )),
        ..Validation::new(Algorithm::RS256)
    };
    validation.set_audience(&[firebase_config.project_id.clone()]);

    let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e);
    let decoded_token = jsonwebtoken::decode::<Claims>(token, &key, &validation);
    decoded_token
}

pub async fn get_firebase_jwks() -> Result<HashMap<String, JWK>, reqwest::Error> {
    let url =
        "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";
    let resp = reqwest::get(url).await?.json::<KeysResponse>().await?;
    Ok(resp.into())
}

pub async fn verify_id_token(
    token: &str,
) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
    let firebase_config = FirebaseConfig::new();
    verify(token, &firebase_config).await
}

pub struct AuthedUserId(UserId);

// TODO: https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
//     : のやり方の方が綺麗なので、こっちにかえる
impl FromRequest for AuthedUserId {
    type Config = ();
    type Error = ApplicationError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let mut bearer_token: Option<String> = None;

        if let Some(auth_val) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_val.to_str() {
                if auth_str.starts_with("bearer") || auth_str.starts_with("Bearer") {
                    let token = auth_str[6..auth_str.len()].trim();
                    bearer_token = Some(token.to_string());
                }
            }
        }

        match bearer_token {
            Some(token) => Box::pin(async move {
                let decoded = verify_id_token(&token).await;
                match decoded {
                    Ok(token_data) => {
                        let uid = token_data.claims.sub;
                        Ok(AuthedUserId(UserId::from_string(uid)))
                    }
                    Err(_) => Err(ApplicationError::AuthError("verify failed!".into())),
                }
            }),
            _ => {
                Box::pin(async move { Err(ApplicationError::AuthError("token not found!".into())) })
            }
        }
    }
}

async fn get<U: 'static + UserUseCase>(
    usecase: web::Data<U>,
    id: web::Path<UserId>,
    authed_id: AuthedUserId,
) -> Result<HttpResponse> {
    (id.into_inner() == authed_id.0)
        .then(|| ())
        .ok_or(ApplicationError::AuthError("User doesn't match!".into()))?;
    Ok(HttpResponse::Ok().json(usecase.find(&authed_id.0).await?))
}

async fn post<U: 'static + UserUseCase>(
    usecase: web::Data<U>,
    req: web::Json<UserCreateRequest>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Created().json(usecase.create(&req).await?))
}

pub trait BuildUserService: AddService {
    fn build_user_service<U: 'static + UserUseCase>(self) -> Self {
        self.add_service(
            web::scope("/users")
                .service(web::resource("/").route(web::post().to(post::<U>)))
                .service(web::resource("/{id}").route(web::get().to(get::<U>))),
        )
    }
}

impl<T: AddService> BuildUserService for T {}
