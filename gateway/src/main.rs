use std::{env, fs::File, io::Read, sync::Arc};

use axum::{body::{to_bytes, Body, Bytes}, extract::{Request, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}, routing::{get, post}, Router};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let config = load_config();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("checkers={}", config.debug_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let client = reqwest::Client::new();
    info!("Created reqwest client.");

    info!("Configuring services.");
    let mut services: Vec<ServiceConfig> = config.services;
    services.push(ServiceConfig { path: String::from("/game"), host: String::from("http://localhost"), port: 8080 });
    services.push(ServiceConfig { path: String::from("/authenticate"), host: String::from("http://localhost"), port: 8080 });
    services.push(ServiceConfig { path: String::from("/register"), host: String::from("http://localhost"), port: 8080 });
    services.push(ServiceConfig { path: String::from("/refresh"), host: String::from("http://localhost"), port: 8080 });

    services.push(ServiceConfig { path: String::from("/app"), host: String::from("http://localhost"), port: 8080 });
    services.push(ServiceConfig { path: String::from("/topic"), host: String::from("http://localhost"), port: 8080 });
    services.push(ServiceConfig { path: String::from("/play"), host: String::from("http://localhost"), port: 8080 });

    let state = AppState { client, services };
    let origins = [config.frontend.parse().unwrap(),];
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_headers(Any)
        .allow_methods(Any);

    let app = Router::new()
        .route("/*path", get(handle))
        .route("/*path", post(handle))
        .with_state(Arc::new(state))
        .layer(cors);

    info!("Initializing router…");
    let host = "0.0.0.0";
    let port = config.port;
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .unwrap();
    info!("Router initialized. Listening on port {}.", port);

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn handle(State(state): State<Arc<AppState>>, req: Request<Body>) -> Response {
    let Some(service) = state.services.iter().find(|serv| req.uri().path().starts_with(&serv.path)) else {
        debug!("No match for '{}'.", req.uri());
        return (StatusCode::NOT_FOUND, "Not found").into_response();
    };
    let client = &state.client;
    handle_service(client, req, service)
        .await
        .into_response()
}

async fn handle_service(client: &Client, req: Request<Body>, service: &ServiceConfig) -> impl IntoResponse {
    debug!("'{}' matched against '{}/**'.", req.uri(), service.path);
    let uri = format!("{}:{}{}", service.host, service.port, req.uri().path());
    debug!("Calling {}.", uri);
    let (parts, body) = req.into_parts();

    let response = client.request(parts.method, uri)
        .version(parts.version)
        .headers(parts.headers)
        .body(to_bytes(body, usize::MAX).await.unwrap())
        .send()
        .await;

    let result = ServiceResult::from(response).await;

    return result
}

enum ServiceResult {
    Response(ServiceResponse),
    Error(String),
}

struct ServiceResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

impl ServiceResult {
    async fn from(request_result: Result<reqwest::Response, reqwest::Error>) -> ServiceResult {
        match request_result {
            Err(err) => {
                ServiceResult::Error(err.to_string())
            },
            Ok(response) => {
                ServiceResult::Response(ServiceResponse::from(response).await)
            },
        }
    }
}

impl ServiceResponse {
    async fn from(response: reqwest::Response) -> ServiceResponse {
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await.unwrap();

        ServiceResponse { status, headers, body }
    }
}

impl IntoResponse for ServiceResult {
    fn into_response(self) -> Response {
        match self {
            ServiceResult::Error(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response(),
            ServiceResult::Response(resp) => (resp.status, resp.headers, resp.body).into_response(),
        }
    }
}

struct AppState {
    client: reqwest::Client,
    services: Vec<ServiceConfig>
}

#[derive(Deserialize, Serialize)]
struct ServiceConfig {
    path: String,
    host: String,
    port: usize,
}

#[derive(Deserialize, Serialize)]
struct Config {
    services: Vec<ServiceConfig>,
    debug_level: Option<String>,
    port: Option<usize>,
    frontend: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config { 
            services: vec![],
            port: None,
            debug_level: None,
            frontend: None,
        } 
    } 
}

struct ConfigFin {
    services: Vec<ServiceConfig>,
    debug_level: String,
    port: usize,
    frontend: String,
}

fn load_config() -> ConfigFin {
    let env_config = load_env_config();
    let config = load_yaml_config("config.yaml");

    ConfigFin {
        debug_level: match (config.debug_level, env_config.debug_level) {
            (_, Some(value)) => value,
            (Some(value), None) => value,
            (None, None) => String::from("debug"),
        },
        port: match (config.port, env_config.port) {
            (_, Some(value)) => value,
            (Some(value), None) => value,
            (None, None) => 8000,
        },
        frontend: match (config.frontend, env_config.frontend) {
            (_, Some(value)) => value,
            (Some(value), None) => value,
            (None, None) => String::from("http://localhost:4200"),
        },
        services: config.services,
    }
}

fn load_yaml_config(path: &str) -> Config {
    debug!("Reading services from yaml file…");
    let file = File::open(path);

    let Ok(mut file) = file else {
        debug!("No yaml configuration found.");
        return Config::default()
    };
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    debug!("Deserializing…");
    let config: Config = serde_yaml::from_str(&content).unwrap();
    config
}

fn load_env_config() -> Config {
    Config {
        debug_level: match env::var("DEBUG_LEVEL") {
            Ok(env) => Some(env),
            _ => None,
        },
        port: match env::var("SERVER_PORT") {
            Ok(env) => match env.parse() {
                Err(_) => None,
                Ok(env) => Some(env),
            },
            _ => None,
        },
        frontend: match env::var("FRONTEND") {
            Ok(env) => Some(env),
            _ => None,
        },
        services: vec![],
    }
}
