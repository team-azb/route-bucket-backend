use actix_web::{ResponseError, http::StatusCode, dev::HttpResponseBuilder, http::header, HttpResponse};
use derive_more::{Display};
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum ApplicationError {
    #[display("Bad Request: {0}")]
    BadRequest(&'static str),

    #[display("Internal Server Error: {0}")]
    InternalServerError(&'static str),
}

impl ResponseError for ApplicationError {
    fn status_code(&self) -> StatusCode {
        match *self {
            ApplicationError::BadRequest(..) => StatusCode::BAD_REQUEST,
            ApplicationError::InternalServerError(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }
}