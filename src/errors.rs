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
