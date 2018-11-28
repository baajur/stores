//! Module containing base_product model for query, insert, update
use std::time::SystemTime;

use serde_json;
use uuid::Uuid;
use validator::Validate;

use stq_static_resources::{Currency, ModerationStatus};
use stq_types::{AttributeId, BaseProductId, BaseProductSlug, CategoryId, ProductId, ProductPrice, StoreId};

use models::validation_rules::*;
use models::{NewProductWithAttributes, Product, ProductWithAttributes, Store};

use schema::base_products;

/// Payload for querying base_products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(Store)]
#[table_name = "base_products"]
pub struct BaseProduct {
    pub id: BaseProductId,
    pub store_id: StoreId,
    pub is_active: bool,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub category_id: CategoryId,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub views: i32,
    pub seo_title: Option<serde_json::Value>,
    pub seo_description: Option<serde_json::Value>,
    pub rating: f64,
    pub slug: BaseProductSlug,
    pub status: ModerationStatus,
    pub kafka_update_no: i32,
    pub currency: Currency,
    pub uuid: Uuid,
}

/// Payload for creating base_products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone, Debug)]
#[table_name = "base_products"]
pub struct NewBaseProduct {
    #[validate(custom = "validate_translation")]
    pub name: serde_json::Value,
    pub store_id: StoreId,
    #[validate(custom = "validate_translation")]
    pub short_description: serde_json::Value,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_title: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_description: Option<serde_json::Value>,
    pub currency: Currency,
    pub category_id: CategoryId,
    #[validate(custom = "validate_slug")]
    pub slug: Option<String>,
    pub uuid: Uuid,
}

/// Payload for creating base product with variants
#[derive(Serialize, Deserialize, Validate, Clone, Debug)]
pub struct NewBaseProductWithVariants {
    #[serde(flatten)]
    pub new_base_product: NewBaseProduct,
    pub variants: Vec<NewProductWithAttributes>,
    pub selected_attributes: Vec<AttributeId>,
}

/// Payload for updating base_products
#[derive(Serialize, Deserialize, Insertable, Validate, AsChangeset, Clone, Debug, Default)]
#[table_name = "base_products"]
pub struct UpdateBaseProduct {
    #[validate(custom = "validate_translation")]
    pub name: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub short_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub long_description: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_title: Option<serde_json::Value>,
    #[validate(custom = "validate_translation")]
    pub seo_description: Option<serde_json::Value>,
    pub currency: Option<Currency>,
    pub category_id: Option<CategoryId>,
    pub rating: Option<f64>,
    #[validate(custom = "validate_slug")]
    pub slug: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticProduct {
    pub id: BaseProductId,
    pub name: serde_json::Value,
    pub short_description: serde_json::Value,
    pub long_description: Option<serde_json::Value>,
    pub views: i32,
    pub rating: Option<f64>,
    pub variants: Vec<ElasticVariant>,
    pub category_id: i32,
    pub matched_variants_ids: Option<Vec<ProductId>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticVariant {
    pub prod_id: ProductId,
    pub discount: Option<f64>,
    pub price: ProductPrice,
    pub attrs: Vec<ElasticAttrValue>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElasticAttrValue {
    pub attr_id: i32,
    pub str_val: Option<String>,
    pub float_val: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseProductWithVariants {
    #[serde(flatten)]
    pub base_product: BaseProduct,
    pub variants: Vec<Product>,
}

impl BaseProductWithVariants {
    pub fn new(base_product: BaseProduct, variants: Vec<Product>) -> Self {
        Self { base_product, variants }
    }
}

#[derive(Debug, Clone)]
pub struct CatalogWithAttributes {
    pub base_product: BaseProduct,
    pub variants: Vec<ProductWithAttributes>,
}

impl CatalogWithAttributes {
    pub fn new(base_product: BaseProduct, variants: Vec<ProductWithAttributes>) -> Self {
        Self { base_product, variants }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ModeratorBaseProductSearchTerms {
    pub name: Option<String>,
    pub store_id: Option<i32>,
    pub state: Option<ModerationStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModeratorBaseProductSearchResults {
    pub base_products: Vec<BaseProduct>,
    pub total_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseProductModerate {
    pub base_product_id: BaseProductId,
    pub status: ModerationStatus,
}
