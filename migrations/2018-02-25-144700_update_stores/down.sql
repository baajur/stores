-- This file should undo anything in `up.sql`
ALTER TABLE stores ALTER COLUMN phone TYPE VARCHAR NOT NULL;
ALTER TABLE stores ALTER COLUMN email TYPE VARCHAR NOT NULL;
ALTER TABLE stores ALTER COLUMN address TYPE VARCHAR NOT NULL;
ALTER TABLE stores DROP COLUMN IF EXISTS default_language;
ALTER TABLE stores DROP COLUMN IF EXISTS slogan;