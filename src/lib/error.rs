use actix_web::{ResponseError, http::StatusCode, dev::HttpResponseBuilder, http::header, HttpResponse};
use derive_more::Display;
use thiserror::Error;

/// ApplicationErrorを持つResult用のエイリアス
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// actix-webを用いて直接リクエストに変換できる自作エラークラス
#[derive(Debug, Display, Error)]
pub enum ApplicationError {
    #[display(fmt = "ValueObjectError: {}", _0)]
    ValueObjectError(&'static str),

    #[display(fmt = "ResourceNotFound: {} {} not found", resource_name, id)]
    ResourceNotFound{ resource_name: &'static str, id: String },
}

impl ResponseError for ApplicationError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ApplicationError::ValueObjectError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ApplicationError::ResourceNotFound{..} => StatusCode::NOT_FOUND,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}