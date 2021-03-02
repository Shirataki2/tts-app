-- Add migration script here
ALTER TABLE user_secret ADD UNIQUE (users_id);
