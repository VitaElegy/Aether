use crate::domain::{
    models::AuthClaims,
    ports::{AuthError, AuthService, UserRepository},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;

pub struct Arg2JwtAuthService {
    user_repo: Arc<dyn UserRepository>,
    jwt_secret: String,
}

impl Arg2JwtAuthService {
    pub fn new(user_repo: Arc<dyn UserRepository>, secret: String) -> Self {
        Self {
            user_repo,
            jwt_secret: secret,
        }
    }
}

#[async_trait]
impl AuthService for Arg2JwtAuthService {
    async fn authenticate(&self, username: &str, password: &str) -> Result<AuthClaims, AuthError> {
        // 1. Fetch User
        let user = self
            .user_repo
            .find_by_username(username)
            .await
            .map_err(AuthError::RepoError)?
            .ok_or(AuthError::InvalidCredentials)?;

        // 2. Verify Password (Argon2)
        let parsed_hash =
            PasswordHash::new(&user.password_hash).map_err(|_| AuthError::InvalidCredentials)?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthError::InvalidCredentials)?;

        // 3. Generate Claims
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = AuthClaims {
            sub: user.id.0.to_string(),
            exp: expiration,
            perms: user.permissions,
        };

        Ok(claims)
    }

    fn verify_token(&self, token: &str) -> Result<AuthClaims, AuthError> {
        let token_data = decode::<AuthClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }

    fn generate_token(&self, claims: &AuthClaims) -> Result<String, AuthError> {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenGenerationError(e.to_string()))
    }
}

// Utility to hash passwords (useful for registration or seeding)
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}
