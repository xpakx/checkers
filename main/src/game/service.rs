use std::sync::Arc;

use axum::{extract::State, response::{IntoResponse, Response}, Json};
use tracing::{debug, info};

use crate::{game::repository::{get_finished_games, get_games, get_requests}, security::UserData, user::repository::get_user, AppState};

pub async fn games(State(state): State<Arc<AppState>>, user: UserData) -> Response {
    info!("List of active games requested…");
    let username = user.username;

    debug!("Trying to get user {} from db…", username);
    let query_result = get_user(&state.db, &username).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let user = query_result.unwrap();

    debug!("Trying to fetch games for user {} from db…", username);
    let query_result = get_games(&state.db, &user.id).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let games = query_result.unwrap();

    return Json(games).into_response()
}

pub async fn archive(State(state): State<Arc<AppState>>, user: UserData) -> Response {
    info!("List of finished games requested…");
    let username = user.username;

    debug!("Trying to get user {} from db…", username);
    let query_result = get_user(&state.db, &username).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let user = query_result.unwrap();

    debug!("Trying to fetch finished games for user {} from db…", username);
    let query_result = get_finished_games(&state.db, &user.id).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let games = query_result.unwrap();

    return Json(games).into_response()
}

pub async fn requests(State(state): State<Arc<AppState>>, user: UserData) -> Response {
    info!("List of game requests requested…");
    let username = user.username;

    debug!("Trying to get user {} from db…", username);
    let query_result = get_user(&state.db, &username).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let user = query_result.unwrap();

    debug!("Trying to fetch game requests for user {} from db…", username);
    let query_result = get_requests(&state.db, &user.id).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let games = query_result.unwrap();

    return Json(games).into_response()
}
