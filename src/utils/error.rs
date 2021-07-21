use actix_web::{http, HttpResponse, HttpResponseBuilder, ResponseError};
use derive_more::Display;
use thiserror::Error;

use crate::hashmap;

/// ApplicationErrorを持つResult用のエイリアス
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// actix-webを用いて直接リクエストに変換できる自作エラークラス
#[derive(Clone, Debug, Display, Error)]
pub enum ApplicationError {
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

    #[display(fmt = "ValueObjectError: {}", _0)]
    ValueObjectError(String),
}

impl ResponseError for ApplicationError {
    fn status_code(&self) -> http::StatusCode {
        match *self {
            ApplicationError::InvalidOperation(..) => http::StatusCode::BAD_REQUEST,
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
