use std::sync::Arc;

use axum::{extract::State, response::{IntoResponse, Response}, Json};
use tracing::{debug, info};

use crate::{game::{repository::{get_finished_games, get_games, get_requests, save_game}, NewGameResponse}, security::UserData, user::repository::get_user, validation::ValidatedForm, AppState};

use super::GameRequest;

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

// TODO
pub async fn new_game(State(state): State<Arc<AppState>>, user: UserData, ValidatedForm(game): ValidatedForm<GameRequest>) -> Response {
    info!("Creating new game requested…");
    let username = user.username;
    let opponent = game.opponent.unwrap();

    debug!("Trying to get user {} from db…", username);
    let query_result = get_user(&state.db, &username).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let user = query_result.unwrap();

    debug!("Trying to get opponent {} from db…", opponent);
    let query_result = get_user(&state.db, &opponent).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let opponent = query_result.unwrap();

    debug!("Trying to add game to db…");
    let query_result = save_game(&state.db, &user.id, &opponent.id).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let id = query_result.unwrap();

    info!("Game {} succesfully created.", id);

    return Json(NewGameResponse{game_id: id}).into_response()
}
