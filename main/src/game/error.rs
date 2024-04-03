use axum::{http::StatusCode, response::{IntoResponse, Response}};

#[derive(Debug)]
pub enum FetchGameError {
    Unknown,
}

impl From<sqlx::Error> for FetchGameError {
    fn from(_error: sqlx::Error) -> Self {
        return FetchGameError::Unknown
    }
}

impl IntoResponse for FetchGameError {
    fn into_response(self) -> Response {
        // TODO
        match self {
            FetchGameError::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, "Database error!"),
        }
        .into_response()
    }
}
