use serde::{Deserialize, Serialize};
use validator::Validate;


pub mod service;
pub mod error;
pub mod repository;

#[derive(Serialize, Deserialize)]
pub enum GameType {
    AI, User,
}

#[derive(Serialize, Deserialize)]
pub enum Rules {
    British,
}

#[derive(Serialize, Deserialize)]
pub enum AIType {
    None, Random, Counting,
}

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GameRequest {
    // TODO
    pub opponent: Option<String>,
    #[serde(rename = "type")]
    #[validate(required(message = "Game type must be specified!"))]
    pub game_type: Option<GameType>,
    #[validate(required(message = "Rule set must be specified!"))]
    pub rules: Option<Rules>,
    pub ai_type: Option<AIType>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewGameResponse {
    pub game_id: i64,
}

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AcceptRequest {
    pub status: InvitationStatus,
}

#[derive(Serialize, Deserialize)]
pub enum InvitationStatus {
    Accepted,
    Rejected,
}
