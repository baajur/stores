use std::convert::From;

use diesel;
use diesel::prelude::*;
use diesel::select;
use diesel::dsl::exists;
use diesel::query_dsl::RunQueryDsl;
use diesel::query_dsl::LoadQuery;
use diesel::pg::PgConnection;

use models::{NewProduct, Product, UpdateProduct};
use models::product::products::dsl::*;
use super::error::Error;
use super::types::{DbConnection, RepoResult};
use repos::acl::Acl;


/// Products repository, responsible for handling products
pub struct ProductsRepoImpl<'a> {
    pub db_conn: &'a DbConnection,
    pub acl: Box<Acl>,
}

pub trait ProductsRepo {
    /// Find specific product by ID
    fn find(&self, product_id: i32) -> RepoResult<Product>;

    /// Verifies product exist
    fn name_exists(&self, name_arg: String) -> RepoResult<bool>;

    /// Find specific product by full name
    fn find_by_name(&self, name_arg: String) -> RepoResult<Product>;

    /// Returns list of products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Product>>;

    /// Creates new product
    fn create(&self, payload: NewProduct) -> RepoResult<Product>;

    /// Updates specific product
    fn update(&self, product_id: i32, payload: UpdateProduct) -> RepoResult<Product>;

    /// Deactivates specific product
    fn deactivate(&self, product_id: i32) -> RepoResult<Product>;
}

impl<'a> ProductsRepoImpl<'a> {
    pub fn new(db_conn: &'a DbConnection, acl: Box<Acl>) -> Self {
        Self { db_conn, acl }
    }


    fn execute_query<T: Send + 'static, U: LoadQuery<PgConnection, T> + Send + 'static>(
        &self,
        query: U,
    ) -> RepoResult<T> {
        query
            .get_result::<T>(&**self.db_conn)
            .map_err(|e| Error::from(e))
    }
}

impl<'a> ProductsRepo for ProductsRepoImpl<'a> {
    /// Find specific product by ID
    fn find(&self, product_id_arg: i32) -> RepoResult<Product> {
        self.execute_query(products.find(product_id_arg))
    }

    /// Verifies product exist
    fn name_exists(&self, name_arg: String) -> RepoResult<bool> {
        self.execute_query(select(exists(products.filter(name.eq(name_arg)))))
    }

    /// Find specific product by full name
    fn find_by_name(&self, name_arg: String) -> RepoResult<Product> {
        let query = products.filter(name.eq(name_arg));

        query
            .first::<Product>(&**self.db_conn)
            .map_err(|e| Error::from(e))
    }


    /// Creates new product
    fn create(&self, payload: NewProduct) -> RepoResult<Product> {
        let query_product = diesel::insert_into(products).values(&payload);
        query_product
            .get_result::<Product>(&**self.db_conn)
            .map_err(Error::from)
    }

    /// Returns list of products, limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> RepoResult<Vec<Product>> {
        let query = products
            .filter(is_active.eq(true))
            .filter(id.gt(from))
            .order(id)
            .limit(count);

        query
            .get_results(&**self.db_conn)
            .map_err(|e| Error::from(e))
    }

    /// Updates specific product
    fn update(&self, product_id_arg: i32, payload: UpdateProduct) -> RepoResult<Product> {
        let filter = products
            .filter(id.eq(product_id_arg))
            .filter(is_active.eq(true));

        let query = diesel::update(filter).set(&payload);
        query
            .get_result::<Product>(&**self.db_conn)
            .map_err(|e| Error::from(e))
    }

    /// Deactivates specific product
    fn deactivate(&self, product_id_arg: i32) -> RepoResult<Product> {
        let filter = products
            .filter(id.eq(product_id_arg))
            .filter(is_active.eq(true));
        let query = diesel::update(filter).set(is_active.eq(false));
        self.execute_query(query)
    }
}
