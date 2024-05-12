use std::env;

use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres;

async fn config() {
    let container = postgres::Postgres::default()
        .with_user("user")
        .with_password("password")
        .with_db_name("checkers")
        .start().await;
    let postgres_ip = container.get_host().await;
    let postgres_port = container.get_host_port_ipv4(5432).await;
    let postgres_url = format!("postgresql://user:password@{}:{}/checkers", postgres_ip, postgres_port);
    

    env::set_var("DEBUG_LEVEL", "info");
    env::set_var("SERVER_PORT", "8080");
    env::set_var("JWT_SECRET", "secret");
    env::set_var("DB_URL", postgres_url);
    env::set_var("RABBIT_URL", "");
}

