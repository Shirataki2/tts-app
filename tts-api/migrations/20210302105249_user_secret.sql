-- Add migration script here
CREATE TABLE user_secret (
    users_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL
)
