use sqlx::{postgres::PgQueryResult, PgPool, Postgres};
use tracing::debug;

use crate::{user::error::{RegistrationError, FetchUserError}, UserModel};

pub async fn save_user(db: &PgPool, username: &String, password: &String) -> Result<PgQueryResult, RegistrationError> {
    sqlx::query("INSERT INTO account (username, password) VALUES ($1, $2)")
        .bind(username)
        .bind(password)
        .execute(db)
        .await
        .map_err(|err: sqlx::Error| { 
            debug!("Cannot add user to db!");
            debug!("{}", err); 
            RegistrationError::from(err)
        })
}

pub async fn get_user(db: &PgPool, username: &String) -> Result<UserModel, FetchUserError> {
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
