use crate::models::auth::{Claims, KEYS};
use jsonwebtoken::{encode, Header};

use super::time::get_unix_timestamp_ms;

pub async fn generate_jwt(account_id: String) -> anyhow::Result<String> {
    let exp = get_unix_timestamp_ms() + 1000 * 60 * 60 * 24 * 30; // 30 days
    let claims = Claims {
        sub: account_id,
        company: "Clutch".to_string(),
        exp: exp as usize,
    };
    encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| anyhow::anyhow!("Error creating token"))
}

pub async fn decode_jwt(token: String) -> anyhow::Result<Claims> {
    let token_data = jsonwebtoken::decode::<Claims>(
        &token,
        &KEYS.decoding,
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|_| anyhow::anyhow!("Error decoding token"))?;
    Ok(token_data.claims)
}

pub async fn validate_jwt_claims(claims: Claims) -> anyhow::Result<()> {
    let now = get_unix_timestamp_ms();
    if claims.exp < now as usize {
        return Err(anyhow::anyhow!("Token expired"));
    }
    Ok(())
}
