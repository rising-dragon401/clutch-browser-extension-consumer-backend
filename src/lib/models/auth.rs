use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::config::settings::{Env, Settings};

pub static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = &Settings::new(Env::Test).unwrap().jwt.key();
    Keys::new(secret.as_bytes())
});

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub company: String,
    pub exp: usize,
}

// #[derive(Debug)]
// pub enum AuthError {
//     WrongCredentials,
//     MissingCredentials,
//     TokenCreation,
//     InvalidToken,
// }

// impl IntoResponse for AuthError {
//     fn into_response(self) -> Response {
//         let (status, error_message) = match self {
//             AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
//             AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
//             AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
//             AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
//         };
//         let body = Json(json!({
//             "error": error_message,
//         }));
//         (status, body).into_response()
//     }
// }
