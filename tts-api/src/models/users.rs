use crate::error::AppError;
use sqlx::{query, query_as, PgPool};

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    pub id: i64,
    pub account_status: i32,
    pub character_count: i64,
    pub character_limit: i64,
}

impl User {
    pub async fn get(pool: &PgPool, id: i64) -> Result<User, AppError> {
        let user = query_as!(User, "SELECT * FROM users WHERE id = $1", id)
            .fetch_one(pool)
            .await?;
        Ok(user)
    }

    pub async fn create(pool: &PgPool, id: i64) -> Result<(), AppError> {
        query!(
            "INSERT INTO users (id, account_status, character_count, character_limit) VALUES ($1, $2, $3, $4)",
            id, 0, 0, 5000,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn get_or_create(pool: &PgPool, id: i64) -> Result<User, AppError> {
        let database_err = match Self::get(pool, id).await {
            Ok(user) => return Ok(user),
            Err(AppError::DatabaseError(e)) => e,
            Err(e) => return Err(e),
        };
        match database_err {
            sqlx::Error::RowNotFound => {}
            e => return Err(e.into()),
        };
        Self::create(pool, id).await?;
        Self::get(pool, id).await
    }

    pub async fn use_capability(pool: &PgPool, id: i64, length: i64) -> Result<(), AppError> {
        let user = Self::get(pool, id).await?;
        let new_capability = user.character_count + length;
        query!("UPDATE users SET character_count = $1", new_capability)
            .execute(pool)
            .await?;
        Ok(())
    }
}
