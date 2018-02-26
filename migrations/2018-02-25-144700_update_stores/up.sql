
ALTER TABLE stores ALTER COLUMN phone TYPE VARCHAR;
ALTER TABLE stores ALTER COLUMN email TYPE VARCHAR;
ALTER TABLE stores ALTER COLUMN address TYPE VARCHAR;
ALTER TABLE stores ADD COLUMN IF NOT EXISTS default_language INTEGER NOT NULL REFERENCES languages (id) DEFAULT 1;
ALTER TABLE stores ADD COLUMN IF NOT EXISTS slogan VARCHAR;
