use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorType {
    BadRequest,
    NotFound,
    Internal,
    DistantServer,
    CacheError,
    Unauthorized,
    MissingRequiredField,
    AlreadyExists,
    Validation,
    PayloadTooLarge,
    Forbidden,
    Test,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppError {
    pub err_type: ErrorType,
    pub message: String,
}

impl AppError {
    pub fn new(message: &str, err_type: ErrorType) -> AppError {
        AppError {
            message: message.to_string(),
            err_type,
        }
    }

    pub fn from_diesel_err(err: diesel::result::Error, context: &str) -> AppError {
        AppError::new(
            format!("{}: {}", context, err.to_string()).as_str(),
            match err {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => ErrorType::BadRequest,
                diesel::result::Error::NotFound => ErrorType::NotFound,
                _ => ErrorType::Internal,
            },
        )
    }
}
