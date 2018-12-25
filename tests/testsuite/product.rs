use std::str::FromStr;
use std::time::SystemTime;

use hyper::header::{Authorization, ContentLength, ContentType};
use hyper::Uri;
use hyper::{Method, Request};

use futures::Future;
use rand::Rng;

use stq_http::request_util::read_body;
use stq_static_resources::Currency;
use stq_types::*;

use common::*;
use stores_lib::models::*;

pub fn create_new_base_product(name: &str, short_description: &str) -> NewBaseProduct {
    NewBaseProduct {
        name: serde_json::from_str(name).unwrap(),
        store_id: StoreId(1),
        short_description: serde_json::from_str(short_description).unwrap(),
        long_description: None,
        seo_title: None,
        seo_description: None,
        currency: Currency::STQ,
        category_id: CategoryId(12),
        slug: Some(rand::thread_rng().gen_ascii_chars().take(10).collect::<String>().to_lowercase()),
        uuid: uuid::Uuid::new_v4(),
        length_cm: Some(60),
        width_cm: Some(40),
        height_cm: Some(20),
        weight_g: Some(100),
    }
}

pub fn create_product(id: ProductId, base_product_id: BaseProductId) -> RawProduct {
    RawProduct {
        id: id,
        base_product_id: base_product_id,
        is_active: true,
        discount: None,
        photo_main: None,
        vendor_code: "vendor code".to_string(),
        cashback: None,
        additional_photos: None,
        price: ProductPrice(1f64),
        currency: Currency::STQ,
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
        pre_order: false,
        pre_order_days: 0,
        kafka_update_no: 0,
        uuid: uuid::Uuid::new_v4(),
    }
}

pub fn create_new_product_with_attributes(base_product_id: BaseProductId) -> NewProductWithAttributes {
    NewProductWithAttributes {
        product: create_new_product(base_product_id),
        attributes: vec![],
    }
}

pub fn create_new_product(base_product_id: BaseProductId) -> NewProductWithoutCurrency {
    NewProductWithoutCurrency {
        base_product_id: Some(base_product_id),
        discount: None,
        photo_main: None,
        vendor_code: "vendor code".to_string(),
        cashback: None,
        additional_photos: None,
        price: ProductPrice(1f64),
        pre_order: Some(false),
        pre_order_days: Some(0),
        uuid: uuid::Uuid::new_v4(),
    }
}

pub fn create_update_product() -> UpdateProduct {
    UpdateProduct {
        discount: None,
        photo_main: None,
        vendor_code: None,
        cashback: None,
        additional_photos: None,
        price: Some(ProductPrice(2f64)),
        currency: Some(Currency::STQ),
        pre_order: Some(false),
        pre_order_days: Some(0),
    }
}

pub fn create_update_product_with_attributes() -> UpdateProductWithAttributes {
    UpdateProductWithAttributes {
        product: Some(create_update_product()),
        attributes: Some(vec![]),
    }
}

static MOCK_BASE_PRODUCT_NAME_JSON: &'static str = r##"[{"lang": "en","text": "Base Product"}]"##;
static MOCK_SHORT_DESCRIPTION_JSON: &'static str = r##"[{"lang": "en","text": "Short Description"}]"##;

#[ignore]
#[test]
fn products_crud() {
    let mut context = setup();

    //create base product
    let mut url = Uri::from_str(&format!("{}/base_products", context.base_url)).unwrap();

    let new_base_product = create_new_base_product(MOCK_BASE_PRODUCT_NAME_JSON, MOCK_SHORT_DESCRIPTION_JSON);
    let mut body: String = serde_json::to_string(&new_base_product).unwrap().to_string();

    let mut req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.headers_mut().set(Authorization("1".to_string()));
    req.set_body(body);

    let mut code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<BaseProduct>(&code);
    assert!(value.is_ok());

    let id = value.unwrap().id;

    //create
    url = Uri::from_str(&format!("{}/products", context.base_url)).unwrap();

    let new_product = create_new_product_with_attributes(id);
    body = serde_json::to_string(&new_product).unwrap().to_string();

    req = Request::new(Method::Post, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.headers_mut().set(Authorization("1".to_string()));
    req.set_body(body);

    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<RawProduct>(&code);
    assert!(value.is_ok());

    let id = value.unwrap().id;

    //read
    url = Uri::from_str(&format!("{}/products/{}", context.base_url, id)).unwrap();

    req = Request::new(Method::Get, url.clone());
    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<RawProduct>(&code);
    assert!(value.is_ok());

    //update
    url = Uri::from_str(&format!("{}/products/{}", context.base_url, id)).unwrap();

    let update_product = create_update_product_with_attributes();
    body = serde_json::to_string(&update_product).unwrap().to_string();

    req = Request::new(Method::Put, url.clone());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    req.headers_mut().set(Authorization("1".to_string()));
    req.set_body(body);

    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<RawProduct>(&code);
    assert!(value.is_ok());

    //delete
    url = Uri::from_str(&format!("{}/products/{}", context.base_url, id)).unwrap();

    req = Request::new(Method::Delete, url.clone());
    req.headers_mut().set(Authorization("1".to_string()));
    code = context
        .core
        .run(context.client.request(req).and_then(|res| read_body(res.body())))
        .unwrap();
    let value = serde_json::from_str::<RawProduct>(&code);
    assert!(value.is_ok());
}