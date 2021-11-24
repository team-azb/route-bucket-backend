use actix_web::{http, HttpResponse, HttpResponseBuilder, ResponseError};
use derive_more::Display;
use thiserror::Error;

use crate::hashmap;

/// ApplicationErrorを持つResult用のエイリアス
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// actix-webを用いて直接リクエストに変換できる自作エラークラス
#[derive(Clone, Debug, Display, Error, PartialEq)]
pub enum ApplicationError {
    #[display(fmt = "AuthError: {}", _0)]
    AuthError(String),

    #[display(fmt = "DataBaseError: {}", _0)]
    DataBaseError(String),

    #[display(fmt = "DomainError: {}", _0)]
    DomainError(String),

    #[display(fmt = "ExternalError: {}", _0)]
    ExternalError(String),

    #[display(fmt = "InvalidOperation: {}", _0)]
    InvalidOperation(&'static str),

    #[display(fmt = "ResourceNotFound: {} {} not found", resource_name, id)]
    ResourceNotFound {
        resource_name: &'static str,
        id: String,
    },

    #[display(fmt = "UseCaseError: {}", _0)]
    UseCaseError(String),

    #[display(fmt = "ValidationError: {}", _0)]
    ValidationError(String),

    #[display(fmt = "ValueObjectError: {}", _0)]
    ValueObjectError(String),
}

impl ResponseError for ApplicationError {
    fn status_code(&self) -> http::StatusCode {
        match *self {
            ApplicationError::AuthError(..) => http::StatusCode::UNAUTHORIZED,
            ApplicationError::InvalidOperation(..) | ApplicationError::ValidationError(..) => {
                http::StatusCode::BAD_REQUEST
            }
            ApplicationError::ResourceNotFound { .. } => http::StatusCode::NOT_FOUND,
            ApplicationError::DataBaseError(..)
            | ApplicationError::DomainError(..)
            | ApplicationError::ExternalError(..)
            | ApplicationError::UseCaseError { .. }
            | ApplicationError::ValueObjectError(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .content_type("text/html; charset=utf-8")
            .json(hashmap! {"message" => self.to_string()})
    }
}

impl From<jsonwebtoken::errors::Error> for ApplicationError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        ApplicationError::AuthError(err.to_string())
    }
}

impl From<reqwest::Error> for ApplicationError {
    fn from(err: reqwest::Error) -> Self {
        ApplicationError::ExternalError(err.to_string())
    }
}

impl From<validator::ValidationErrors> for ApplicationError {
    fn from(err: validator::ValidationErrors) -> Self {
        ApplicationError::ValidationError(err.to_string())
    }
}
