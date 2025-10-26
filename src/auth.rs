use axum::{
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    uid: i64,
    exp: usize,
}

pub fn extract_uid_from_headers(headers: &HeaderMap) -> Result<i64, StatusCode> {
    // Check if running on localhost (development mode)
    if let Some(host) = headers.get("host") {
        if let Ok(host_str) = host.to_str() {
            if host_str.starts_with("localhost") {
                return Ok(1); // Default uid for localhost
            }
        }
    }

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
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )?;

    Ok(token_data.claims.uid)
}