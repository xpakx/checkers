use core::fmt;
use std::sync::Arc;

use axum::{async_trait, extract::{rejection::FormRejection, FromRequest, Request, State}, http::StatusCode, response::{IntoResponse, Response}, routing::post, Form, Json, Router};
use jsonwebtoken::{decode, DecodingKey, Validation};
use security::TokenClaims;
use sqlx::{postgres::PgPoolOptions, PgPool, Postgres};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use validator::Validate;

use crate::security::{get_token, verify_password};
use crate::user::{service::register, AuthResponse};

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

async fn login(
    State(state): State<Arc<AppState>>,
    Form(user): Form<AuthRequest>) -> impl IntoResponse {
    info!("Authentication requested…");
    let username = user.username.unwrap();
    let password = user.password.unwrap();

    let user_db = get_user(&state.db, &username).await;

    if let Err(err) = user_db {
        return err.into_response()
    };
    let user_db = user_db.unwrap();

    verify_password(username, &user_db.password, password)
}

async fn get_user(db: &PgPool, username: &String) -> Result<UserModel, FetchUserError> {
    debug!("Trying to get user {} from db…", username);
    let result = sqlx::query_as::<Postgres, UserModel>(
        "SELECT * FROM account WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot get user from db!");
            debug!("{}", err); 
            FetchUserError::from(err)
        });
    match result {
        Ok(None) => Err(FetchUserError::NoUser),
        Ok(Some(user)) => Ok(user),
        Err(err) => Err(err),
    }
}

#[derive(Debug)]
enum FetchUserError {
    NoUser,
    Unknown,
}

impl From<sqlx::Error> for FetchUserError {
    fn from(_error: sqlx::Error) -> Self {
        return FetchUserError::Unknown
    }
}


impl IntoResponse for FetchUserError {
    fn into_response(self) -> Response {
        // TODO
        match self {
            FetchUserError::NoUser => {
                (StatusCode::BAD_REQUEST, "No such user!")
            }
            FetchUserError::Unknown => (StatusCode::INTERNAL_SERVER_ERROR, "Database error!"),
        }
        .into_response()
    }
}


#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct AuthRequest {
    #[validate(required(message = "Username cannot be empty"))]
    username: Option<String>,

    #[validate(required(message = "Password cannot be empty"))]
    password: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[allow(non_snake_case)]
struct UserModel {
    id: i32,
    username: String,
    password: String,
}

async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Form(request): Form<RefreshRequest>) -> impl IntoResponse {
    info!("Refreshing token requested…");
    let token = request.token.unwrap();
    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::default(),
        );

    let claims = match claims {
        Ok(c) => c,
        Err(_) => return (StatusCode::BAD_REQUEST, "Malformed token!").into_response(),
    };

    if claims.claims.exp < (chrono::Utc::now().timestamp() as usize) {
        return (StatusCode::BAD_REQUEST, "Expired token").into_response()
    }

    if !claims.claims.refresh {
        return (StatusCode::BAD_REQUEST, "Not a refresh token").into_response()
    }

    let username = claims.claims.sub;
    let user_db = get_user(&state.db, &username).await;

    if let Err(err) = user_db {
        return err.into_response()
    };

    let refresh_token = get_token(&username, true).unwrap_or(String::from(""));
    let token = get_token(&username, false).unwrap_or(String::from(""));
    let response = AuthResponse { username, token, refresh_token, moderator_role: false };
    Json(response).into_response()
}


#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
struct RefreshRequest {
    #[validate(required(message = "Token cannot be empty"))]
    token: Option<String>,
}
