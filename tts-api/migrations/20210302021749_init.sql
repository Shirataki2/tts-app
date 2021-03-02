-- Add migration script here
CREATE TABLE users
(
    id BIGINT NOT NULL,
    account_status INT NOT NULL,
    character_count BIGINT NOT NULL,
    character_limit BIGINT NOT NULL,
    CONSTRAINT users_pk PRIMARY KEY (id)
);
