use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Deserialize;
use serde_json::json;
use std::fmt;

/// Represents the HTTP error.
///
/// # Fields
///
/// * `error_status_code` - The status code of the error.
/// * `error_message` - The message of the error.
#[derive(Debug, Deserialize)]
pub struct HttpError {
    pub error_status_code: u16,
    pub error_message: String,
}

impl HttpError {
    /// Creates a new `HttpError`.
    ///
    /// # Arguments
    ///
    /// * `error_status_code` - The status code of the error.
    /// * `error_message` - The message of the error.
    ///
    /// # Returns
    ///
    /// A new `HttpError`.
    pub fn new(error_status_code: u16, error_message: String) -> HttpError {
        HttpError {
            error_status_code,
            error_message,
        }
    }
}

impl fmt::Display for HttpError {
    /// Formats the error message.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    ///
    /// # Returns
    ///
    /// A `fmt::Result`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.error_message.as_str())
    }
}

impl ResponseError for HttpError {
    /// Creates an error response.
    ///
    /// # Arguments
    ///
    /// * `self` - The `HttpError`.
    ///
    /// # Returns
    ///
    /// An `HttpResponse` with the error message.
    /// If the status code is not valid, a `HttpResponse` with the status code `INTERNAL_SERVER_ERROR` will be returned.
    fn error_response(&self) -> HttpResponse {
        let status_code = match StatusCode::from_u16(self.error_status_code) {
            Ok(status_code) => status_code,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error_message = match status_code.as_u16() < 500 {
            true => self.error_message.clone(),
            false => "Internal server error".to_string(),
        };

        HttpResponse::build(status_code).json(json!({ "message": error_message }))
    }
}
