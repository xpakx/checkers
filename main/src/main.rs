use core::fmt;
use std::sync::Arc;

use axum::{async_trait, extract::{rejection::FormRejection, FromRequest, Request}, http::StatusCode, response::{IntoResponse, Response}, routing::post, Form, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use validator::Validate;

use crate::user::{service::{register, login, refresh_token}, AuthResponse};

mod security;
mod user;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "checkers=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = "postgresql://root:password@localhost:5432/checkers";
    info!("Connecting to database…");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();
 
    info!("Connection to database established.");
    
    info!("Applying migrations…");
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap();

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/register", post(register))
        .route("/authenticate", post(login))
        .route("/refresh", post(refresh_token))
        .with_state(Arc::new(state));

    info!("Initializing router…");
    let host = "0.0.0.0";
    let port = 8080;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    info!("Router initialized. Listening on port {}.", port);

    axum::serve(listener, app)
        .await
        .unwrap();
}

pub struct AppState {
    db: PgPool,
}

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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[allow(non_snake_case)]
struct UserModel {
    id: i32,
    username: String,
    password: String,
}
