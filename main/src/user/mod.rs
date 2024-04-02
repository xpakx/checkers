use serde::{Deserialize, Serialize};
use validator::Validate;

pub mod repository; 
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

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    #[validate(required(message = "Token cannot be empty"))]
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserRequest {
    #[validate(
        length(min = 5, max=15, message = "Username length must be between 5 and 15"),
        required(message = "Username cannot be empty"),
        custom(function = "validate_not_ai_username")
        )]
        username: Option<String>,

        #[validate(
            required(message = "Password cannot be empty"),
            must_match(other = "password_re", message="Passwords don't match!")
            )]
            password: Option<String>,
            password_re: Option<String>,
}

fn validate_not_ai_username(username: &Option<String>) -> Result<(), validator::ValidationError> {
    match username {
        None => Ok(()),
        Some(username) => {
            if username.starts_with("AI") {
                return Err(validator::ValidationError::new("Username cannot start with \"AI\"!"));
            }
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest {
    #[validate(required(message = "Username cannot be empty"))]
    pub username: Option<String>,

    #[validate(required(message = "Password cannot be empty"))]
    pub password: Option<String>,
}
