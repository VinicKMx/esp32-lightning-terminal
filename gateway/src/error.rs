use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use terminal_protocol::{ErrorResponse, RequestId};
use uuid::Uuid;

use crate::{auth::AuthError, provider::PaymentProviderError};

#[derive(Debug)]
pub enum ApiError {
    Auth(AuthError),
    BadRequest(String),
    NotFound(String),
    Provider(PaymentProviderError),
}

impl From<AuthError> for ApiError {
    fn from(error: AuthError) -> Self {
        Self::Auth(error)
    }
}

impl From<PaymentProviderError> for ApiError {
    fn from(error: PaymentProviderError) -> Self {
        Self::Provider(error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Auth(error) => (StatusCode::UNAUTHORIZED, "unauthorized", error.to_string()),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            Self::Provider(error) => (
                StatusCode::BAD_GATEWAY,
                "payment_provider_error",
                error.to_string(),
            ),
        };

        let request_id = match RequestId::new(Uuid::new_v4().to_string()) {
            Ok(request_id) => request_id,
            Err(_) => match RequestId::new("request-id-generation-failed") {
                Ok(request_id) => request_id,
                Err(_) => {
                    return (
                        status,
                        Json(serde_json::json!({
                            "code": code,
                            "message": message,
                            "request_id": "request-id-generation-failed"
                        })),
                    )
                        .into_response();
                }
            },
        };

        (status, Json(ErrorResponse::new(code, message, request_id))).into_response()
    }
}
