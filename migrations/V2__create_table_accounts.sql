CREATE TABLE IF NOT EXISTS accounts (
    id             TEXT    PRIMARY KEY,
    email          TEXT    NOT NULL,
    wallet_address TEXT    NOT NULL,
    eoa_address    TEXT    NOT NULL,
    eoa_private_address    TEXT    NOT NULL,
    updated_at     INTEGER NOT NULL
);