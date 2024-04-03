use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres};
use tracing::debug;

use crate::game::error::FetchGameError;

pub async fn get_games(db: &PgPool, user_id: &i32) -> Result<Vec<GameDetails>, FetchGameError> {
    sqlx::query_as::<Postgres, GameDetails>("SELECT g.*, a1.username AS user, a2.username AS opponent  
                                          FROM game g 
                                          LEFT JOIN account a1 ON a1.id = g.user_id 
                                          LEFT JOIN account a2 ON a2.id = g.opponent_id 
                                          WHERE (g.user_id = $1 OR g.opponent_id = $1) AND g.accepted = true AND g.finished = false")
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot get games from db!");
            debug!("{}", err); 
            FetchGameError::from(err)
        })
}

pub async fn get_finished_games(db: &PgPool, user_id: &i32) -> Result<Vec<GameDetails>, FetchGameError> {
    sqlx::query_as::<Postgres, GameDetails>("SELECT g.*, a1.username AS user, a2.username AS opponent  
                                          FROM game g 
                                          LEFT JOIN account a1 ON a1.id = g.user_id 
                                          LEFT JOIN account a2 ON a2.id = g.opponent_id 
                                          WHERE (g.user_id = $1 OR g.opponent_id = $1) AND g.accepted = true AND g.finished = true")
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot get games from db!");
            debug!("{}", err); 
            FetchGameError::from(err)
        })
}

pub async fn get_requests(db: &PgPool, user_id: &i32) -> Result<Vec<GameDetails>, FetchGameError> {
    sqlx::query_as::<Postgres, GameDetails>("SELECT g.*, a1.username AS user, a2.username AS opponent  
                                          FROM game g 
                                          LEFT JOIN account a1 ON a1.id = g.user_id 
                                          LEFT JOIN account a2 ON a2.id = g.opponent_id 
                                          WHERE g.opponent_id = $1 AND g.accepted = false AND g.rejected = false")
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot get games from db!");
            debug!("{}", err); 
            FetchGameError::from(err)
        })
}

// TODO
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct GameModel {
    id: i32,
    accepted: bool,
    rejected: bool,

    finished: bool,
    user_id: i32,
    opponent_id: i32,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct GameDetails {
    id: i32,
    accepted: bool,
    rejected: bool,

    finished: bool,
    user: String,
    opponent: String,
}
