ALTER TABLE base_products DROP COLUMN IF EXISTS currency;
ALTER TABLE base_products ADD COLUMN currency_id INTEGER NOT NULL DEFAULT 6;

ALTER TABLE products DROP COLUMN IF EXISTS currency;
ALTER TABLE products ADD COLUMN currency_id INTEGER;
