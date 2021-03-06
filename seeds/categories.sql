DELETE FROM prod_attr_values;
DELETE FROM products;
DELETE FROM base_products;
DELETE FROM categories;

ALTER SEQUENCE categories_id_seq RESTART WITH 1;

INSERT INTO categories (uuid, name, meta_field, parent_id) VALUES
(uuid_generate_v4(), '[{"lang": "en", "text": "Jewelry & Accessories"}, {"lang": "ru", "text": "Ювелирные изделия и аксессуары"}]' ,NULL	,NULL),
(uuid_generate_v4(), '[{"lang": "en", "text": "Clothing & Shoes"}, {"lang": "ru", "text": "Одежда и обувь"}]' ,NULL	,NULL),
(uuid_generate_v4(), '[{"lang": "en", "text": "Accessories"}, {"lang": "ru", "text": "Аксессуары"}]' ,NULL,	1),
(uuid_generate_v4(), '[{"lang": "en", "text": "Bags & purses"}, {"lang": "ru", "text": "Сумки и чехлы"}]' ,NULL,	1),
(uuid_generate_v4(), '[{"lang": "en", "text": "Necklaces"}, {"lang": "ru", "text": "Ожерелья"}]' ,NULL,	1),
(uuid_generate_v4(), '[{"lang": "en", "text": "Bracelets"}, {"lang": "ru", "text": "Браслеты"}]' ,NULL,	1),
(uuid_generate_v4(), '[{"lang": "en", "text": "Womens"}, {"lang": "ru", "text": "Женский"}]' ,NULL,	2),
(uuid_generate_v4(), '[{"lang": "en", "text": "Mens"}, {"lang": "ru", "text": "Мужской"}]' ,NULL,	2),
(uuid_generate_v4(), '[{"lang": "en", "text": "Hair Accessories"}, {"lang": "ru", "text": "Аксессуары для волос"}]' ,NULL,	3),
(uuid_generate_v4(), '[{"lang": "en", "text": "Sunglasses & Eyewear"}, {"lang": "ru", "text": "Солнцезащитные очки и очки"}]' ,NULL,	3),
(uuid_generate_v4(), '[{"lang": "en", "text": "Belts & Suspenders"}, {"lang": "ru", "text": "Ремни и подтяжки"}]' ,NULL,	3),
(uuid_generate_v4(), '[{"lang": "en", "text": "Keychains & Lanyards"}, {"lang": "ru", "text": "Брелки и ремешки"}]' ,NULL,	3),
(uuid_generate_v4(), '[{"lang": "en", "text": "Umbrellas & Rain Accessories"}, {"lang": "ru", "text": "Зонтики и аксессуары"}]' ,NULL,	3),
(uuid_generate_v4(), '[{"lang": "en", "text": "Backpacks"}, {"lang": "ru", "text": "Рюкзаки"}]' ,NULL,	4),
(uuid_generate_v4(), '[{"lang": "en", "text": "Phone Case"}, {"lang": "ru", "text": "Чехол для телефона"}]' ,NULL,	4),
(uuid_generate_v4(), '[{"lang": "en", "text": "Beaded Bracelets"}, {"lang": "ru", "text": "Браслеты из бисера"}]' ,NULL,	6),
(uuid_generate_v4(), '[{"lang": "en", "text": "Cuff Bracelets"}, {"lang": "ru", "text": "Браслеты с манжетами"}]' ,NULL,	6),
(uuid_generate_v4(), '[{"lang": "en", "text": "Dresses"}, {"lang": "ru", "text": "Платья"}]' ,NULL,	7),
(uuid_generate_v4(), '[{"lang": "en", "text": "Jackets & Coats"}, {"lang": "ru", "text": "Куртки и пальто"}]' ,NULL,	7),
(uuid_generate_v4(), '[{"lang": "en", "text": "Shirts"}, {"lang": "ru", "text": "Рубашки"}]' ,NULL,	8),
(uuid_generate_v4(), '[{"lang": "en", "text": "Sweaters"}, {"lang": "ru", "text": "Свитера"}]' ,NULL,	8),
(uuid_generate_v4(), '[{"lang": "en", "text": "Chains"}, {"lang": "ru", "text": "Цепочки"}]' ,NULL,	5),
(uuid_generate_v4(), '[{"lang": "en", "text": "Pendants"}, {"lang": "ru", "text": "Подвески"}]' ,NULL,	5);
