-- added a new column called ip of type INET
ALTER TABLE token_transfers ADD COLUMN ip INET;

-- update all timestamps from naive to UTC
ALTER TABLE token_transfers 
ALTER COLUMN created_at TYPE TIMESTAMPTZ USING created_at AT TIME ZONE 'UTC',
ALTER COLUMN updated_at TYPE TIMESTAMPTZ USING updated_at AT TIME ZONE 'UTC';