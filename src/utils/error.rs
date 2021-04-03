use actix_web::{dev, http, HttpResponse, ResponseError};
use derive_more::Display;
use thiserror::Error;

use crate::hashmap;

/// ApplicationErrorを持つResult用のエイリアス
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// actix-webを用いて直接リクエストに変換できる自作エラークラス
#[derive(Debug, Display, Error)]
pub enum ApplicationError {
    #[display(fmt = "DataBaseError: {}", _0)]
    DataBaseError(&'static str),

    #[display(fmt = "DomainError: {}", _0)]
    DomainError(&'static str),

    #[display(fmt = "ValueObjectError: {}", _0)]
    ValueObjectError(String),

    #[display(fmt = "ResourceNotFound: {} {} not found", resource_name, id)]
    ResourceNotFound {
        resource_name: &'static str,
        id: String,
    },
}

impl ResponseError for ApplicationError {
    fn status_code(&self) -> http::StatusCode {
        match *self {
            ApplicationError::DataBaseError(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::DomainError(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::ValueObjectError(..) => http::StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::ResourceNotFound { .. } => http::StatusCode::NOT_FOUND,
        }
    }
    fn error_response(&self) -> HttpResponse {
        dev::HttpResponseBuilder::new(self.status_code())
            .set_header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
            .json(hashmap! {"message" => self.to_string()})
    }
}
