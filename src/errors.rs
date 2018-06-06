use failure::Error;
use hyper::StatusCode;
use serde_json;
use validator::ValidationErrors;

use stq_http::errors::{Codeable, PayloadCarrier};

#[derive(Debug, Fail)]
pub enum ControllerError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Parse error")]
    Parse(String),
    #[fail(display = "Validation error")]
    Validate(ValidationErrors),
    #[fail(display = "Server is refusing to fullfil the reqeust")]
    Forbidden,
    #[fail(display = "Server is refusing to fullfil the reqeust: {}", _0)]
    Connection(Error),
    #[fail(display = "Server is refusing to fullfil the reqeust: {}", _0)]
    ElasticSearch(Error),
}

impl Codeable for ControllerError {
    fn code(&self) -> StatusCode {
        match *self {
            ControllerError::NotFound => StatusCode::NotFound,
            ControllerError::Validate(_) => StatusCode::BadRequest,
            ControllerError::Parse(_) => StatusCode::UnprocessableEntity,
            ControllerError::Connection(_) | ControllerError::ElasticSearch(_) => StatusCode::InternalServerError,
            ControllerError::Forbidden => StatusCode::Forbidden,
        }
    }
}

impl PayloadCarrier for ControllerError {
    fn payload(&self) -> Option<serde_json::Value> {
        match *self {
            ControllerError::Validate(ref e) => serde_json::to_value(e.clone()).ok(),
            _ => None,
        }
    }
}
