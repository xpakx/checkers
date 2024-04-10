use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use bcrypt::hash;
use jsonwebtoken::{decode, DecodingKey, Validation};
use tracing::{debug, info};

use crate::{security::{get_token, verify_password, TokenClaims}, user::{repository::{get_user, save_user}, AuthResponse}, validation::ValidatedJson, AppState};

use super::{RefreshRequest, UserRequest, AuthRequest};

pub async fn register(State(state): State<Arc<AppState>>, ValidatedJson(user): ValidatedJson<UserRequest>) -> Response {
    info!("Creating new user requested…");
    let username = user.username.unwrap();
    let password = hash(user.password.unwrap(), 10).unwrap();

    debug!("Trying to add user {} to db…", username);
    let query_result = save_user(&state.db, &username, &password).await;

    if let Err(err) = query_result {
        return err.into_response()
    }

    info!("User {} succesfully created.", username);

    let refresh_token = get_token(&username, true, &state.jwt).unwrap_or(String::from(""));
    let token = get_token(&username, false, &state.jwt).unwrap_or(String::from(""));
    let response = AuthResponse { username, token, refresh_token, moderator_role: false };
    return Json(response).into_response()
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    ValidatedJson(user): ValidatedJson<AuthRequest>) -> impl IntoResponse {
    info!("Authentication requested…");
    let username = user.username.unwrap();
    let password = user.password.unwrap();

    let user_db = get_user(&state.db, &username).await;

    if let Err(err) = user_db {
        return err.into_response()
    };
    let user_db = user_db.unwrap();

    verify_password(username, &user_db.password, password, &state.jwt)
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    ValidatedJson(request): ValidatedJson<RefreshRequest>) -> impl IntoResponse {
    info!("Refreshing token requested…");
    let token = request.token.unwrap();
    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(state.jwt.as_ref()),
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

    let refresh_token = get_token(&username, true, &state.jwt).unwrap_or(String::from(""));
    let token = get_token(&username, false, &state.jwt).unwrap_or(String::from(""));
    let response = AuthResponse { username, token, refresh_token, moderator_role: false };
    Json(response).into_response()
}
