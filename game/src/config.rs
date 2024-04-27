use std::{env, fs::File, io::Read};

use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Deserialize, Serialize)]
struct Config {
    debug_level: Option<String>,
    port: Option<usize>,
    jwt_secret: Option<String>,
    rabbit: Option<String>,
    redis: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config { 
            debug_level: None,
            port: None,
            jwt_secret: None,
            rabbit: None,
            redis: None,
        } 
    } 
}

pub struct ConfigFin {
    pub debug_level: String,
    pub port: usize,
    pub jwt_secret: String,
    pub rabbit: String,
    pub redis: String,
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
        jwt_secret: match env::var("JWT_SECRET") {
            Ok(env) => Some(env),
            _ => None,
        },
        redis: match env::var("REDIS_URL") {
            Ok(env) => Some(env),
            _ => None,
        },
        rabbit: match env::var("RABBIT_URL") {
            Ok(env) => Some(env),
            _ => None,
        },
    }
}

pub fn get_config() -> ConfigFin {
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
            (None, None) => 8080,
        },
        jwt_secret: match (config.jwt_secret, env_config.jwt_secret) {
            (_, Some(value)) => value,
            (Some(value), None) => value,
            (None, None) => String::from("secret"),
        },
        redis: match (config.redis, env_config.redis) {
            (_, Some(value)) => value,
            (Some(value), None) => value,
            (None, None) => String::from("redis://default:redispw@localhost:6379"),
        },
        rabbit: match (config.rabbit, env_config.rabbit) {
            (_, Some(value)) => value,
            (Some(value), None) => value,
            (None, None) => String::from("amqp://guest:guest@localhost:5672"),
        },
    }
}
