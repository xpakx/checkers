use std::{env, fs::File, io::Read};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Config {
    rabbit: Option<String>,
}

impl Default for Config {
    fn default() -> Config {
        Config { 
            rabbit: None,
        } 
    } 
}

pub struct ConfigFin {
    pub rabbit: String,
}

fn load_yaml_config(path: &str) -> Config {
    let file = File::open(path);
    let Ok(mut file) = file else {
        return Config::default()
    };
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let config: Config = serde_yaml::from_str(&content).unwrap();
    config
}

fn load_env_config() -> Config {
    Config {
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
        rabbit: match (config.rabbit, env_config.rabbit) {
            (_, Some(value)) => value,
            (Some(value), None) => value,
            (None, None) => String::from("amqp://guest:guest@localhost:5672"),
        },
    }
}
