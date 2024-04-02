use axum::{async_trait, extract::FromRequestParts, http::{request::Parts, StatusCode}, response::{IntoResponse, Response}, Json};
use bcrypt::verify;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::AuthResponse;

pub fn get_token(username: &String, refresh: bool) -> Result<String, String> {
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let duration = match refresh {
        true => chrono::Duration::days(90),
        false => chrono::Duration::minutes(10),
    };
    let exp = (now + duration).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: String::from(username),
        exp,
        iat,
        refresh,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    ).map_err(|_| String::from("Couldn't create token"))
}

pub fn verify_password(username: String, real_password: &String, password: String) -> Response {
    match verify(password, real_password).unwrap() {
        false => (StatusCode::UNAUTHORIZED, "Wrong password").into_response(),
        true => {
            let refresh_token = get_token(&username, true).unwrap_or(String::from(""));
            let token = get_token(&username, false).unwrap_or(String::from(""));
            let response = AuthResponse { username, token, refresh_token, moderator_role: false };
            Json(response).into_response()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
    pub refresh: bool,
}

#[allow(dead_code)]
pub struct UserData {
    username: String,
}


#[derive(Debug)]
pub enum TokenError {
    Expired,
    Malformed,
    RefreshToken,
    MissingToken,
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        // TODO
        match self {
            TokenError::Expired => (StatusCode::FORBIDDEN, "Token expired"),
            TokenError::Malformed => (StatusCode::FORBIDDEN, "Malformed token"),
            TokenError::RefreshToken => (StatusCode::FORBIDDEN, "Cannot use refresh token for auth"),
            TokenError::MissingToken => (StatusCode::FORBIDDEN, "No token"),
        }
        .into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserData
where
    S: Send + Sync,
{
    type Rejection = TokenError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get("Authorization").ok_or(TokenError::MissingToken)?;
        let token = match auth_header.to_str() {
            Ok(header_value) => {
                if header_value.starts_with("Bearer ") {
                    header_value.trim_start_matches("Bearer ").to_string()
                } else {
                    return Err(TokenError::Malformed);
                }
            }
            Err(_) => return Err(TokenError::Malformed),
        };

        let claims = decode::<TokenClaims>(
            &token,
            &DecodingKey::from_secret("secret".as_ref()),
            &Validation::default(),
            );

        let claims = match claims {
            Ok(c) => c,
            Err(_) => return Err(TokenError::Malformed),
        };

        if claims.claims.exp < (chrono::Utc::now().timestamp() as usize) {
            return Err(TokenError::Expired);
        }

        if claims.claims.refresh {
            return Err(TokenError::RefreshToken);
        }

        return Ok(UserData { username: claims.claims.sub });
    }
}
