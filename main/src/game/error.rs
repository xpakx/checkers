use axum::{http::StatusCode, response::{IntoResponse, Response}};

#[derive(Debug)]
pub enum GameError {
    Unknown,
    NotFound,
    NotOwner,
    AlreadyRejected,
    AlreadyAccepted,
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
            GameError::NotOwner => (StatusCode::FORBIDDEN, "You cannot change this request!"),
            GameError::AlreadyRejected => (StatusCode::BAD_REQUEST, "Request already rejected!"),
            GameError::AlreadyAccepted => (StatusCode::BAD_REQUEST, "Request already acccepted!"),
        }
        .into_response()
    }
}

#[derive(Debug)]
pub enum MoveError {
    Unknown,
    NoGame,
}

impl From<sqlx::Error> for MoveError {
    fn from(error: sqlx::Error) -> Self {
        let err = error.to_string();
        if err.contains("fk_game_id") {
            return MoveError::NoGame
        }
        return MoveError::Unknown
    }
}

impl IntoResponse for MoveError {
    fn into_response(self) -> Response {
        // TODO
        match self {
            MoveError::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, "Database error!"),
            MoveError::NoGame => (StatusCode::NOT_FOUND, "Game not found!"),
        }
        .into_response()
    }
}
