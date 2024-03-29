use core::fmt;
use std::sync::Arc;

use axum::{async_trait, extract::{rejection::FormRejection, FromRequest, Request, State}, http::StatusCode, response::{IntoResponse, Response}, routing::post, Form, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use validator::Validate;

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
    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();
 
    info!("Connection to database established.");
    
    info!("Applying migrations...");
    sqlx::migrate!()
        .run(&pool)
        .await
        .unwrap();

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/authenticate", post(register))
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

async fn register(State(state): State<Arc<AppState>>, ValidatedForm(user): ValidatedForm<UserRequest>) -> Response {
    info!("Creating new user requested…");
    let username = user.username.unwrap();

    debug!("Trying to add user {} to db...", username);
    let query_result =
        sqlx::query("INSERT INTO account (username, password) VALUES ($1, $2)")
        .bind(&username)
        .bind(&user.password) // TODO: hashing password
        .execute(&state.db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("{}", err); 
            RegistrationError::from(err)
        });

    if let Err(err) = query_result {
        debug!("cannot add user to db!");
        return err.into_response()
    }

    info!("User {} succesfully created.", username);
    return "Hello world".into_response()
}

enum RegistrationError {
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

#[allow(dead_code)]
struct AppState {
    db: PgPool,
}

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserRequest {
    #[validate(
        length(min = 5, max=15, message = "Username length must be between 5 and 15"),
        required(message = "Username cannot be empty"),
        custom(function = "validate_not_ai_username")
    )]
    username: Option<String>,

    #[validate(
        required(message = "Password cannot be empty"),
        must_match(other = "password_re", message="Passwords don't match!")
    )]
    password: Option<String>,
    password_re: Option<String>,
}

fn validate_not_ai_username(username: &Option<String>) -> Result<(), validator::ValidationError> {
    match username {
        None => Ok(()),
        Some(username) => {
            if username.starts_with("AI") {
                return Err(validator::ValidationError::new("Username cannot start with \"AI\"!"));
            }
            Ok(())
        }
    }
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
