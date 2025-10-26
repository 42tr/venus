use axum::{
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub uid: i64,
    pub username: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(uid: i64, username: String) -> Self {
        Self {
            sub: uid.to_string(),
            uid,
            username,
            exp: (Utc::now() + Duration::days(7)).timestamp() as usize, // Token expires in 7 days
        }
    }
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

pub fn generate_jwt_token(uid: i64, username: String) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(uid, username);
    let secret = "your-secret-key-change-this-in-production";
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn extract_uid_from_headers(headers: &HeaderMap) -> Result<i64, StatusCode> {
    // Extract JWT token from Authorization header
    let token = extract_token(headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate JWT token
    validate_jwt_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(header[7..].to_string())
            } else {
                None
            }
        })
        .or_else(|| {
            // Try to get token from cookie
            headers
                .get("cookie")
                .and_then(|header| header.to_str().ok())
                .and_then(|cookies| {
                    cookies
                        .split(';')
                        .find_map(|cookie| {
                            let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
                            if parts.len() == 2 && parts[0] == "token" {
                                Some(parts[1].to_string())
                            } else {
                                None
                            }
                        })
                })
        })
}

fn validate_jwt_token(token: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let secret = "your-secret-key-change-this-in-production";
    
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )?;

    Ok(token_data.claims.uid)
}