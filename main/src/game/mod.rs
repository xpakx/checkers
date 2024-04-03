use serde::{Deserialize, Serialize};
use validator::Validate;

pub mod service;
pub mod error;
mod repository;

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GameRequest {
    // TODO
    pub opponent: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewGameResponse {
    pub game_id: usize,
}
