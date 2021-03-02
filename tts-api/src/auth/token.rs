use std::fmt::{self, Formatter};

use pwhash::bcrypt;
use rand::Rng;
use sqlx::{query, PgPool};

use crate::error::AppError;

const CHARSET: &[u8] = b"ABCDEF0123456789";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Token(String);

impl Token {
    pub fn new(s: &str) -> Token {
        Token(s.to_string())
    }

    pub fn generate(length: usize) -> Token {
        let mut rng = rand::thread_rng();
        let pw: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        Token(pw)
    }

    pub async fn register(&self, pool: &PgPool, user_id: i64) -> Result<(), AppError> {
        let digested = bcrypt::hash(&self.0)?;
        query!(
            r#"
                INSERT INTO user_secret values ($1, $2)
                ON CONFLICT (users_id)
                DO UPDATE SET users_id = $1, token = $2
            "#,
            user_id,
            digested
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn verify(&self, pool: &PgPool, user_id: i64) -> Result<bool, AppError> {
        let digested = query!(
            r#"
                SELECT token from user_secret
                WHERE users_id = $1
            "#,
            user_id
        )
        .fetch_one(pool)
        .await?
        .token;
        Ok(bcrypt::verify(&self.0, &digested))
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
