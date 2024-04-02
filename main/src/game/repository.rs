use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres};
use tracing::debug;

pub async fn get_games(db: &PgPool, user_id: &i32) -> Result<Vec<GameModel>, String> {
    sqlx::query_as::<Postgres, GameModel>("SELECT * FROM game g WHERE (g.user.id = $1 OR G.opponent.id = $1) AND g.accepted = true AND g.finished = false")
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot add user to db!");
            debug!("{}", err); 
            err.to_string()
        })
}

pub async fn get_finished_games(db: &PgPool, user_id: &i32) -> Result<Vec<GameModel>, String> {
    sqlx::query_as::<Postgres, GameModel>("SELECT * FROM game g WHERE (g.user.id = $1 OR G.opponent.id = $1) AND g.accepted = true AND g.finished = true")
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot add user to db!");
            debug!("{}", err); 
            err.to_string()
        })
}

pub async fn get_requests(db: &PgPool, user_id: &i32) -> Result<Vec<GameModel>, String> {
    sqlx::query_as::<Postgres, GameModel>("SELECT * FROM game g WHERE G.opponent.id = $1 AND g.accepted = false AND g.rejected = false")
        .bind(user_id)
        .fetch_all(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot add user to db!");
            debug!("{}", err); 
            err.to_string()
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
}
