CREATE TABLE IF NOT EXISTS verifications (
    id         TEXT    PRIMARY KEY,
    email      TEXT    NOT NULL,
    code       TEXT    NOT NULL,
    expires_at INTEGER NOT NULL
);