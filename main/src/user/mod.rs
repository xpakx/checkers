use serde::{Deserialize, Serialize};

mod repository; 
pub mod error;
pub mod service;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub username: String,
    pub token: String,
    pub refresh_token: String,
    pub moderator_role: bool,
}
