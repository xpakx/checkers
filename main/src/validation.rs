use core::fmt;

use axum::{async_trait, extract::{rejection::{FormRejection, JsonRejection}, FromRequest, Request}, http::StatusCode, response::{IntoResponse, Response}, Form, Json};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
T: DeserializeOwned + Validate,
S: Send + Sync,
Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

#[derive(Debug)]
pub enum ValidationError {
    ValidationError(validator::ValidationErrors),
    AxumFormRejection(JsonRejection),
}

impl From<validator::ValidationErrors> for ValidationError {
    fn from(error: validator::ValidationErrors) -> Self {
        ValidationError::ValidationError(error)
    }
}

impl From<JsonRejection> for ValidationError {
    fn from(error: JsonRejection) -> Self {
        ValidationError::AxumFormRejection(error)
    }
}

impl ValidationError {
    fn to_string(self) -> String {
        // TODO
        match self {
            ValidationError::ValidationError(err) => {
                format!("Validation Error: {}", err)
            }
            ValidationError::AxumFormRejection(err) => {
                format!("Axum Form Rejection: {:?}", err)
            }
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        // TODO
        match self {
            ValidationError::ValidationError(_) => {
                let message = format!("Input validation error: {}", self.to_string());
                (StatusCode::BAD_REQUEST, message)
            }
            ValidationError::AxumFormRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        }
        .into_response()
    }
}

