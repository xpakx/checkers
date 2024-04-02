use axum::{http::StatusCode, response::{IntoResponse, Response}};

pub enum RegistrationError {
    DuplicatedUsername,
    Unknown,
}

impl From<sqlx::Error> for RegistrationError {
    fn from(error: sqlx::Error) -> Self {
        let err = error.to_string();
        if err.contains("duplicate key") && err.contains("username") {
            return RegistrationError::DuplicatedUsername
        }
        return RegistrationError::Unknown
    }
}

impl IntoResponse for RegistrationError {
    fn into_response(self) -> Response {
        // TODO
        match self {
            RegistrationError::DuplicatedUsername => {
                (StatusCode::BAD_REQUEST, "Username already exists!")
            }
            RegistrationError::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, "Database error!"),
        }
        .into_response()
    }
}

