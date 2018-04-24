use diesel::result::Error as DieselError;

use failure::Error;

use stq_http::errors::ControllerError;

use repos::error::RepoError;
use validator::ValidationErrors;

#[derive(Debug, Fail)]
pub enum ServiceError {
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Rollback")]
    Rollback,
    #[fail(display = "Validation error: {}", _0)]
    Validate(ValidationErrors),
    #[fail(display = "Parse error: {}", _0)]
    Parse(String),
    #[fail(display = "R2D2 connection error: {}", _0)]
    Connection(Error),
    #[fail(display = "Diesel transaction error: {}", _0)]
    Transaction(Error),
    #[fail(display = "Repo error: {}", _0)]
    Database(Error),
    #[fail(display = "Http client error: {}", _0)]
    HttpClient(String),
    #[fail(display = "Email already exists: [{}]", _0)]
    EmailAlreadyExists(String),
    #[fail(display = "Incorrect email or password")]
    IncorrectCredentials,
    #[fail(display = "Unauthorized: {}", _0)]
    Unauthorized(String),
    #[fail(display = "Unknown error: {}", _0)]
    Unknown(String),
}

impl From<RepoError> for ServiceError {
    fn from(err: RepoError) -> Self {
        error!("Repo error occured: '{}'.", err);
        match err {
            RepoError::NotFound => ServiceError::NotFound,
            RepoError::Rollback => ServiceError::Rollback,
            RepoError::ContstaintViolation(e) => ServiceError::Database(RepoError::ContstaintViolation(e).into()),
            RepoError::MismatchedType(e) => ServiceError::Database(RepoError::MismatchedType(e).into()),
            RepoError::Connection(e) => ServiceError::Database(RepoError::Connection(e).into()),
            RepoError::Unknown(e) => ServiceError::Database(RepoError::Unknown(e).into()),
            RepoError::Unauthorized(res, act) => {
                ServiceError::Unauthorized(format!("Unauthorized access: Resource: {}, Action: {}", res, act))
            }
        }
    }
}

impl From<DieselError> for ServiceError {
    fn from(err: DieselError) -> Self {
        error!("Diesel error occured: '{}'.", err);
        ServiceError::Transaction(err.into())
    }
}

impl From<ServiceError> for ControllerError {
    fn from(e: ServiceError) -> Self {
        error!("Service error occured: '{}'.", e);
        match e {
            ServiceError::NotFound => ControllerError::NotFound,
            ServiceError::Rollback => ControllerError::BadRequest(ServiceError::Rollback.into()),
            ServiceError::Validate(valid_err) => ControllerError::Validate(valid_err),
            ServiceError::Unauthorized(msg) => ControllerError::Forbidden(ServiceError::Unauthorized(msg).into()),
            ServiceError::Parse(msg) => ControllerError::UnprocessableEntity(ServiceError::Parse(msg).into()),
            ServiceError::Database(msg) => ControllerError::InternalServerError(ServiceError::Database(msg).into()),
            ServiceError::HttpClient(msg) => ControllerError::InternalServerError(ServiceError::HttpClient(msg).into()),
            ServiceError::EmailAlreadyExists(msg) => ControllerError::BadRequest(ServiceError::EmailAlreadyExists(msg).into()),
            ServiceError::IncorrectCredentials => ControllerError::BadRequest(ServiceError::IncorrectCredentials.into()),
            ServiceError::Connection(msg) => ControllerError::InternalServerError(ServiceError::Connection(msg).into()),
            ServiceError::Transaction(msg) => ControllerError::InternalServerError(ServiceError::Transaction(msg).into()),
            ServiceError::Unknown(msg) => ControllerError::InternalServerError(ServiceError::Unknown(msg).into()),
        }
    }
}
