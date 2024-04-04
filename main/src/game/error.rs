use axum::{http::StatusCode, response::{IntoResponse, Response}};

#[derive(Debug)]
pub enum GameError {
    Unknown,
    NotFound,
}

impl From<sqlx::Error> for GameError {
    fn from(_error: sqlx::Error) -> Self {
        return GameError::Unknown
    }
}

impl IntoResponse for GameError {
    fn into_response(self) -> Response {
        // TODO
        match self {
            GameError::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, "Database error!"),
            GameError::NotFound => (StatusCode::NOT_FOUND, "Game not found!"),
        }
        .into_response()
    }
}
