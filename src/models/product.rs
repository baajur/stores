//! Module containg product model for query, insert, update
use std::time::SystemTime;

use validator::Validate;
use diesel::prelude::*;


use super::Language;
use super::Store;
use super::authorization::*;
use repos::types::DbConnection;
use models::store::stores::dsl as Stores;



/// diesel table for products
table! {
    products (id) {
        id -> Integer,
        store_id -> Integer,
        is_active -> Bool,
        name -> VarChar,
        short_description -> VarChar,
        long_description -> Nullable<VarChar>,
        price -> Double,
        currency_id -> Integer,
        discount -> Nullable<Float>,
        category -> Nullable<Integer>,
        photo_main -> Nullable<VarChar>,
        vendor_code -> Nullable<VarChar>,
        cashback -> Nullable<Float>,
        default_language -> Varchar,
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}


/// Payload for querying products
#[derive(Debug, Serialize, Deserialize, Associations, Queryable, Clone, Identifiable)]
#[belongs_to(Store)]
pub struct Product {
    pub id: i32,
    pub store_id: i32,
    pub is_active: bool,
    pub name: String,
    pub short_description: String,
    pub long_description: Option<String>,
    pub price: f64,
    pub currency_id: i32,
    pub discount: Option<f32>,
    pub category: Option<i32>,
    pub photo_main: Option<String>,
    pub vendor_code: Option<String>,
    pub cashback: Option<f32>,
    pub default_language: Language,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

/// Payload for creating products
#[derive(Serialize, Deserialize, Insertable, Validate, Clone)]
#[table_name = "products"]
pub struct NewProduct {
    pub name: String,
    pub store_id: i32,
    pub currency_id: i32,
    pub short_description: String,
    pub long_description: Option<String>,
    pub price: f64,
    pub discount: Option<f32>,
    pub category: Option<i32>,
    pub photo_main: Option<String>,
    pub vendor_code: Option<String>,
    pub cashback: Option<f32>,
    pub default_language: Language,
}

/// Payload for updating products
#[derive(Serialize, Deserialize, Insertable, AsChangeset)]
#[table_name = "products"]
pub struct UpdateProduct {
    pub name: String,
    pub currency_id: Option<i32>,
    pub short_description: Option<String>,
    pub long_description: Option<Option<String>>,
    pub price: Option<f64>,
    pub discount: Option<Option<f32>>,
    pub category: Option<Option<i32>>,
    pub photo_main: Option<Option<String>>,
    pub vendor_code: Option<Option<String>>,
    pub cashback: Option<Option<f32>>,
    pub default_language: Option<Language>,
}

impl WithScope for Product {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(conn) = conn {
                    Stores::stores
                        .find(self.store_id)
                        .get_result::<Store>(&**conn)
                        .and_then(|store: Store| Ok(store.user_id == user_id))
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}

impl WithScope for NewProduct {
    fn is_in_scope(&self, scope: &Scope, user_id: i32, conn: Option<&DbConnection>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(conn) = conn {
                    Stores::stores
                        .find(self.store_id)
                        .get_result::<Store>(&**conn)
                        .and_then(|store: Store| Ok(store.user_id == user_id))
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}