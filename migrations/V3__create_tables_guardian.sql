CREATE TABLE IF NOT EXISTS guardians (
    id             TEXT    PRIMARY KEY,
    email          TEXT    NOT NULL,
    account_id     TEXT        NULL,
    wallet_address TEXT        NULL
);

CREATE TABLE IF NOT EXISTS nominations (
    id          TEXT    PRIMARY KEY,
    email       TEXT    NOT NULL,
    account_id  TEXT    NOT NULL,
    guardian_id TEXT    NOT NULL,
    status      TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS account_guardians (
    id          TEXT  PRIMARY KEY,
    guardian_id TEXT  NOT NULL,
    account_id  TEXT  NOT NULL,
    status      TEXT  NOT NULL
);

CREATE TABLE IF NOT EXISTS guardian_settings (
    id          TEXT  PRIMARY KEY,
    account_id  TEXT  NOT NULL,
    signers     TEXT  NOT NULL
);