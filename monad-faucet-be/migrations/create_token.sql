CREATE TABLE tokens (
    id BIGSERIAL PRIMARY KEY,
    created_by TEXT NOT NULL,
    token_type token_type NOT NULL,
    address TEXT NOT NULL UNIQUE,
    logo_url TEXT NOT NULL,
    chain_id INTEGER NOT NULL,
    symbol TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    decimals INTEGER NOT NULL,
    withdraw_limit TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
