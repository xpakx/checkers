use core::fmt;

use axum::{async_trait, extract::{rejection::FormRejection, FromRequest, Request}, http::StatusCode, response::{IntoResponse, Response}, Form};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedForm<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedForm<T>
where
T: DeserializeOwned + Validate,
S: Send + Sync,
Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedForm(value))
    }
}

#[derive(Debug)]
pub enum ValidationError {
    ValidationError(validator::ValidationErrors),
    AxumFormRejection(FormRejection),
}

impl From<validator::ValidationErrors> for ValidationError {
    fn from(error: validator::ValidationErrors) -> Self {
        ValidationError::ValidationError(error)
    }
}

impl From<FormRejection> for ValidationError {
    fn from(error: FormRejection) -> Self {
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
                let message = format!("Input validation error: [{self}]");
                (StatusCode::BAD_REQUEST, message)
            }
            ValidationError::AxumFormRejection(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        }
        .into_response()
    }
}

