use crate::models::auth::{AuthError, LoginResponse, UserInfo};
use crate::models::session::Session;
use crate::models::user::User;
use crate::providers::sqlite::SQLiteProvider;
use chrono::Utc;
use jsonwebtoken::{encode, Header, EncodingKey};
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn login(
    sqlite: Arc<SQLiteProvider>,
    email: &str,
    password: &str,
    jwt_secret: &str,
) -> Result<LoginResponse, AuthError> {
    let user = sqlite
        .query_one(
            "SELECT * FROM users WHERE email = ?",
            &[email.into()],
            User::from_row,
        )?
        .ok_or(AuthError::UserNotFound)?;

    let password_valid = bcrypt::verify(password, &user.password_hash)
        .map_err(|e| AuthError::PasswordHashError(Box::new(e)))?;
    
    if !password_valid {
        return Err(AuthError::InvalidCredentials);
    }

    let now = Utc::now();
    let access_expires = now + chrono::Duration::hours(1);
    let refresh_expires = now + chrono::Duration::days(7);

    let access_claims = JwtClaims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        exp: access_expires.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let access_token = encode(&Header::default(), &access_claims, &EncodingKey::from_secret(jwt_secret.as_bytes()))?;

    let refresh_claims = JwtClaims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        exp: refresh_expires.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let refresh_token = encode(&Header::default(), &refresh_claims, &EncodingKey::from_secret(jwt_secret.as_bytes()))?;

    let session = Session {
        id: Uuid::new_v4(),
        user_id: user.id,
        access_token: access_token.clone(),
        refresh_token: refresh_token.clone(),
        expires_at: refresh_expires,
        created_at: now,
    };

    sqlite.execute_with_params(
        "INSERT INTO sessions (id, user_id, access_token, refresh_token, expires_at, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            session.id,
            session.user_id,
            session.access_token,
            session.refresh_token,
            session.expires_at,
            session.created_at,
        ],
    )?;

    Ok(LoginResponse {
        status: "ok".to_string(),
        user: UserInfo {
            id: user.id,
            email: user.email,
        },
        access_token,
        refresh_token,
    })
}