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
