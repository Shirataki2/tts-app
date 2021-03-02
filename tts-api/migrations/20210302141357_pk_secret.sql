-- Add migration script here
ALTER TABLE user_secret ADD PRIMARY KEY (token);
