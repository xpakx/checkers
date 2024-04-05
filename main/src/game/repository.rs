use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgQueryResult, PgPool, Postgres};
use tracing::debug;
use chrono;

use crate::game::error::GameError;

pub async fn get_games(db: &PgPool, user_id: &i32) -> Result<Vec<GameDetails>, GameError> {
    sqlx::query_as::<Postgres, GameDetails>("SELECT g.*, a1.username AS user, a2.username AS opponent  
                                          FROM game g 
                                          LEFT JOIN account a1 ON a1.id = g.user_id 
                                          LEFT JOIN account a2 ON a2.id = g.opponent_id 
                                          WHERE (g.user_id = $1 OR g.opponent_id = $1) AND g.invitation = 1 AND g.status = 0")
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot get games from db!");
            debug!("{}", err); 
            GameError::from(err)
        })
}

pub async fn get_finished_games(db: &PgPool, user_id: &i32) -> Result<Vec<GameDetails>, GameError> {
    sqlx::query_as::<Postgres, GameDetails>("SELECT g.*, a1.username AS user, a2.username AS opponent  
                                          FROM game g 
                                          LEFT JOIN account a1 ON a1.id = g.user_id 
                                          LEFT JOIN account a2 ON a2.id = g.opponent_id 
                                          WHERE (g.user_id = $1 OR g.opponent_id = $1) AND g.invitation = 1 AND g.status > 0")
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot get games from db!");
            debug!("{}", err); 
            GameError::from(err)
        })
}

pub async fn get_requests(db: &PgPool, user_id: &i32) -> Result<Vec<GameDetails>, GameError> {
    sqlx::query_as::<Postgres, GameDetails>("SELECT g.*, a1.username AS user, a2.username AS opponent  
                                          FROM game g 
                                          LEFT JOIN account a1 ON a1.id = g.user_id 
                                          LEFT JOIN account a2 ON a2.id = g.opponent_id 
                                          WHERE g.opponent_id = $1 AND g.invitation = 0")
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot get games from db!");
            debug!("{}", err); 
            GameError::from(err)
        })
}

pub async fn save_game(db: &PgPool, game: GameModel) -> Result<i32, GameError> {
    let result = sqlx::query_scalar("INSERT INTO game (user_id, opponent_id, invitation, status) VALUES ($1, $2, $3, $4) RETURNING id")
        .bind(&game.user_id)
        .bind(&game.opponent_id)
        .bind(&game.invitation)
        .bind(&game.status)
        .fetch_one(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot add game to db!");
            debug!("{}", err); 
            GameError::from(err)
        });

    match result {
        Ok(None) => Err(GameError::Unknown),
        Ok(Some(id)) => Ok(id),
        Err(err) => Err(err),
    }
}

pub async fn get_game(db: &PgPool, id: &i32) -> Result<GameModel, GameError> {
    let result = sqlx::query_as::<Postgres, GameModel>("SELECT * FROM game WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot get game from db!");
            debug!("{}", err); 
            GameError::from(err)
        });

    match result {
        Ok(None) => Err(GameError::NotFound),
        Ok(Some(id)) => Ok(id),
        Err(err) => Err(err),
    }
}

pub async fn get_game_details(db: &PgPool, id: &i32) -> Result<GameDetails, GameError> {
    let result = sqlx::query_as::<Postgres, GameDetails>("SELECT g.*, a1.username AS user, a2.username AS opponent  
                                          FROM game g 
                                          LEFT JOIN account a1 ON a1.id = g.user_id 
                                          LEFT JOIN account a2 ON a2.id = g.opponent_id 
                                          WHERE g.id = $1")
        .bind(id)
        .fetch_optional(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot get game from db!");
            debug!("{}", err); 
            GameError::from(err)
        });

    match result {
        Ok(None) => Err(GameError::NotFound),
        Ok(Some(id)) => Ok(id),
        Err(err) => Err(err),
    }
}

pub async fn change_invitation_status(db: &PgPool, id: &i32, status: InvitationStatus) -> Result<PgQueryResult, GameError> {
    sqlx::query("UPDATE games SET invitation = $1 WHERE id = $2")
        .bind(status as i16)
        .bind(id)
        .execute(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot save update to db!");
            debug!("{}", err); 
            GameError::from(err)
        })
}

pub async fn get_moves(db: &PgPool, id: &i32) -> Result<Vec<MoveModel>, GameError> {
    sqlx::query_as::<Postgres, MoveModel>("SELECT * 
                                          FROM moves 
                                          WHERE game_id = ?1 
                                          ORDER BY timestamp ASC")
        .bind(id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Db error");
            debug!("{}", err); 
            GameError::from(err)
        })
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[repr(i16)]
pub enum GameType {
    User = 0,
    AI = 1,
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[repr(i16)]
pub enum RuleSet {
    British = 0,
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[repr(i16)]
pub enum AIType {
    None = 0,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "smallint")]
#[repr(i16)]
pub enum InvitationStatus {
    Issued = 0,
    Accepted = 1,
    Rejected = 2,
}

#[derive(Serialize, Deserialize, sqlx::Type)]
#[repr(i16)]
pub enum GameStatus {
    NotFinished = 0,
    Won = 1,
    Lost = 2,
    Drawn = 3,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct GameModel {
    pub id: Option<i32>,
    pub invitation: InvitationStatus,
    pub game_type: GameType,
    pub ruleset: RuleSet,
    pub ai_type: AIType,
    pub status: GameStatus,

    pub current_state: String,
    pub user_starts: bool,
    pub user_turn: bool,
    pub user_id: i32,
    pub opponent_id: i32,
}

impl Default for GameModel {
    fn default() -> GameModel {
        GameModel { 
            id: None,
            invitation: InvitationStatus::Issued,
            game_type: GameType::User,
            ruleset: RuleSet::British,
            ai_type: AIType::None,
            status: GameStatus::NotFinished,
            current_state: String::from("xxxxxxxxxxxx........oooooooooooo"), // xX=black, oO=white, .=empty
            user_starts: true,
            user_turn: true,
            user_id: 0,
            opponent_id: 0,
        } 
    } 
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct GameDetails {
    pub id: i32,
    pub invitation: InvitationStatus,
    pub game_type: GameType,
    pub ruleset: RuleSet,
    pub ai_type: AIType,
    pub status: GameStatus,

    pub current_state: String,
    pub user_starts: bool,
    pub user_turn: bool,
    pub user_id: i32,
    pub opponent_id: Option<i32>,
    pub user: String,
    pub opponent: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct MoveModel {
    pub id: Option<i32>,
    pub x: i32,
    pub y: i32,
    pub current_state: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,

    pub game_id: i32,
}
