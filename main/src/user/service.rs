use std::sync::Arc;

use axum::{extract::State, response::{Response, IntoResponse}, Json};
use bcrypt::hash;
use tracing::{debug, info};

use crate::{security::get_token, user::{repository::save_user, AuthResponse}, AppState, UserRequest, ValidatedForm};

pub async fn register(State(state): State<Arc<AppState>>, ValidatedForm(user): ValidatedForm<UserRequest>) -> Response {
    info!("Creating new user requested…");
    let username = user.username.unwrap();
    let password = hash(user.password.unwrap(), 10).unwrap();

    debug!("Trying to add user {} to db…", username);
    let query_result = save_user(&state.db, &username, &password).await;

    if let Err(err) = query_result {
        return err.into_response()
    }

    info!("User {} succesfully created.", username);

    let refresh_token = get_token(&username, true).unwrap_or(String::from(""));
    let token = get_token(&username, false).unwrap_or(String::from(""));
    let response = AuthResponse { username, token, refresh_token, moderator_role: false };
    return Json(response).into_response()
}
