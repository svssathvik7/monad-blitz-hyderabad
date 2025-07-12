CREATE TYPE token_type AS ENUM ('ERC20', 'NATIVE');

CREATE TABLE token_transfers (
    id BIGSERIAL PRIMARY KEY,
    token_address TEXT NOT NULL,
    token_type token_type NOT NULL,
    tx_hash TEXT NOT NULL UNIQUE,
    from_address TEXT NOT NULL,
    to_address TEXT NOT NULL,
    amount TEXT NOT NULL,
    chain_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);