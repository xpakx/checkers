use std::sync::Arc;

use axum::{extract::{Path, State}, response::{IntoResponse, Response}, Json};
use tracing::{debug, info};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{game::{self, error::GameError, repository::{change_invitation_status, get_finished_games, get_game, get_game_details, get_games, get_moves, get_requests, save_game, AIType, GameModel, GameResponse, GameType, InvitationStatus, RuleSet}, NewGameResponse}, security::UserData, user::repository::get_user, validation::ValidatedJson, AppState};

use super::{AcceptRequest, GameRequest};

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
    let games: Vec<GameResponse> = query_result.unwrap().iter().map(|game| GameResponse::from(game)).collect();

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
    let games: Vec<GameResponse> = query_result.unwrap().iter().map(|game| GameResponse::from(game)).collect();

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
    let games: Vec<GameResponse> = query_result.unwrap().iter().map(|game| GameResponse::from(game)).collect();

    return Json(games).into_response()
}

pub async fn new_game(State(state): State<Arc<AppState>>, user: UserData, ValidatedJson(game): ValidatedJson<GameRequest>) -> Response {
    info!("Creating new game requested…");
    let username = user.username;
    let ruleset = match game.rules.unwrap() {
        game::Rules::British => RuleSet::British,
    };
    let game_type = match game.game_type.unwrap() {
        game::GameType::User => GameType::User,
        game::GameType::AI => GameType::AI,
    };
    let ai_type = match game.ai_type {
        None => AIType::None,
        Some(ai) => match ai {
            game::AIType::None => AIType::None,
            game::AIType::Random => AIType::Random,
        }
    };
    let invitation = match game_type {
        GameType::AI => InvitationStatus::Accepted,
        GameType::User => InvitationStatus::Issued,
    };
    let seed = chrono::Utc::now().timestamp() as u64;
    let mut rng = StdRng::seed_from_u64(seed);
    let user_starts = rng.gen_bool(0.5);

    debug!("Trying to get user {} from db…", username);
    let query_result = get_user(&state.db, &username).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let user = query_result.unwrap();

    let opponent_id = match game.opponent {
        None => None,
        Some(id) => {
            debug!("Trying to get opponent {} from db…", id);
            let query_result = get_user(&state.db, &id).await;

            if let Err(err) = query_result {
                return err.into_response()
            }
            let opponent = query_result.unwrap();
            Some(opponent.id)
        }
    };

    debug!("Trying to add game to db…");
    let query_result = save_game(&state.db, 
        GameModel { 
            user_id: user.id, 
            opponent_id,
            ruleset,
            game_type,
            ai_type,
            invitation,
            user_starts,
            user_turn: user_starts,
            ..Default::default() 
        }).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let id = query_result.unwrap();

    info!("Game {} succesfully created.", id);

    return Json(NewGameResponse{game_id: id}).into_response()
}

pub async fn accept_request(State(state): State<Arc<AppState>>, user: UserData, Path(id): Path<i64>, ValidatedJson(request): ValidatedJson<AcceptRequest>) -> Response {
    info!("Creating new game requested…");
    let username = user.username;
    let status = match request.status {
        game::InvitationStatus::Accepted => InvitationStatus::Accepted,
        game::InvitationStatus::Rejected => InvitationStatus::Rejected,
    };

    debug!("Trying to get user {} from db…", username);
    let query_result = get_user(&state.db, &username).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let user = query_result.unwrap();

    debug!("Trying to get game {} from db…", id);
    let query_result = get_game(&state.db, &id).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let game = query_result.unwrap();
    let Some(opponent_id) = game.opponent_id else {
        return GameError::NotOwner.into_response()
    };
    if opponent_id != user.id {
        return GameError::NotOwner.into_response()
    }
    if game.invitation != InvitationStatus::Issued {
        return match game.invitation {
            InvitationStatus::Accepted => GameError::AlreadyAccepted,
            InvitationStatus::Rejected => GameError::AlreadyRejected,
            InvitationStatus::Issued => unreachable!(),
        }.into_response()
    }

    debug!("Trying to update invitation status…");
    let query_result = change_invitation_status(&state.db, &id, status).await;

    if let Err(err) = query_result {
        return err.into_response()
    }

    info!("Game {} succesfully updated.", id);

    return Json(NewGameResponse{game_id: id}).into_response()
}

pub async fn game(State(state): State<Arc<AppState>>, _user: UserData, Path(id): Path<i64>) -> Response {
    info!("Getting game requested…");

    debug!("Trying to get game {} from db…", id);
    let query_result = get_game_details(&state.db, &id).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let game = query_result.unwrap();
    let game = GameResponse::from(&game);

    return Json(game).into_response()
}

pub async fn moves(State(state): State<Arc<AppState>>, _user: UserData, Path(id): Path<i64>) -> Response {
    info!("List of moves requested…");

    debug!("Trying to fetch moves for game {} from db…", id);
    let query_result = get_moves(&state.db, &id).await;

    if let Err(err) = query_result {
        return err.into_response()
    }
    let games = query_result.unwrap();

    return Json(games).into_response()
}
