
ALTER TABLE stores 
    ALTER COLUMN address DROP NOT NULL,
    ALTER COLUMN email DROP NOT NULL,
    ALTER COLUMN phone DROP NOT NULL;
ALTER TABLE stores ADD COLUMN language_id INTEGER NOT NULL DEFAULT 1;
ALTER TABLE stores ADD COLUMN slogan VARCHAR;
ALTER TABLE stores DROP COLUMN currency_id;
ALTER TABLE stores ADD COLUMN currency_id INTEGER NOT NULL DEFAULT 1;
