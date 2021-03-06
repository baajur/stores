use diesel;
use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_types::Bool;
use diesel::Connection;
use errors::Error;
use failure::Error as FailureError;

use stq_types::{AttributeId, AttributeValueId, BaseProductId, ProductId, UserId};

use super::acl;
use models::authorization::*;
use models::{BaseProductRaw, NewProdAttr, ProdAttr, Store, UpdateProdAttr};
use repos::legacy_acl::*;
use repos::types::{RepoAcl, RepoResult};
use schema::base_products::dsl as BaseProducts;
use schema::prod_attr_values::dsl::*;
use schema::stores::dsl as Stores;

/// ProductAttrs repository, responsible for handling prod_attr_values
pub struct ProductAttrsRepoImpl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> {
    pub db_conn: &'a T,
    pub acl: Box<RepoAcl<ProdAttr>>,
}

#[derive(Debug, Clone, Default)]
pub struct ProductAttrsSearchTerms {
    pub attr_id: Option<AttributeId>,
    pub attr_value_id: Option<AttributeValueId>,
}

pub trait ProductAttrsRepo {
    /// Find product attributes by product ID
    fn find_all_attributes(&self, product_id_arg: ProductId) -> RepoResult<Vec<ProdAttr>>;

    /// Find product attributes by base_product ID
    fn find_all_attributes_by_base(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<ProdAttr>>;

    /// Creates new product_attribute
    fn create(&self, payload: NewProdAttr) -> RepoResult<ProdAttr>;

    /// Updates specific product_attribute
    fn update(&self, payload: UpdateProdAttr) -> RepoResult<ProdAttr>;

    /// Finds many product attributes by search terms
    fn find_many(&self, search_terms: ProductAttrsSearchTerms) -> RepoResult<Vec<ProdAttr>>;

    /// Delete all attributes values from product
    fn delete_all_attributes(&self, product_id_arg: ProductId) -> RepoResult<Vec<ProdAttr>>;

    /// Delete all attributes values from product not in the list
    fn delete_all_attributes_not_in_list(&self, product_id_arg: ProductId, attr_values: Vec<i32>) -> RepoResult<Vec<ProdAttr>>;

    /// Delete attribute value
    fn delete(&self, id_arg: i32) -> RepoResult<ProdAttr>;

    /// Delete attribute values by base_product ID and attribute ID
    fn delete_by_attribute_id(&self, base_product_id: BaseProductId, attr_id: AttributeId) -> RepoResult<()>;

    /// Delete attribute values by base_product ID
    fn delete_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<()>;
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductAttrsRepoImpl<'a, T> {
    pub fn new(db_conn: &'a T, acl: Box<RepoAcl<ProdAttr>>) -> Self {
        Self { db_conn, acl }
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> ProductAttrsRepo
    for ProductAttrsRepoImpl<'a, T>
{
    /// Find specific product_attributes by product ID
    fn find_all_attributes(&self, product_id_arg: ProductId) -> RepoResult<Vec<ProdAttr>> {
        debug!("Find all attributes of product id {}.", product_id_arg);
        let query = prod_attr_values.filter(prod_id.eq(product_id_arg)).order(attr_id);

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|prod_attrs_res: Vec<ProdAttr>| {
                for prod_attr in &prod_attrs_res {
                    acl::check(&*self.acl, Resource::ProductAttrs, Action::Read, self, Some(&prod_attr))?;
                }
                Ok(prod_attrs_res)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Find specific product_attributes by product id: {} error occurred",
                    product_id_arg
                ))
                .into()
            })
    }

    /// Find product attributes by base_product ID
    fn find_all_attributes_by_base(&self, base_product_id_arg: BaseProductId) -> RepoResult<Vec<ProdAttr>> {
        debug!("Find all attributes of base_product id {}.", base_product_id_arg);
        let query = prod_attr_values.filter(base_prod_id.eq(base_product_id_arg)).order(attr_id);

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|prod_attrs_res: Vec<ProdAttr>| {
                for prod_attr in &prod_attrs_res {
                    acl::check(&*self.acl, Resource::ProductAttrs, Action::Read, self, Some(&prod_attr))?;
                }
                Ok(prod_attrs_res)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Find specific product_attributes by base_product id: {} error occurred",
                    base_product_id_arg
                ))
                .into()
            })
    }

    /// Creates new product_attribute
    fn create(&self, payload: NewProdAttr) -> RepoResult<ProdAttr> {
        debug!("Create new product attribute {:?}.", payload);
        let query_product_attribute = diesel::insert_into(prod_attr_values).values(&payload);
        query_product_attribute
            .get_result::<ProdAttr>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|prod_attr| {
                acl::check(&*self.acl, Resource::ProductAttrs, Action::Create, self, Some(&prod_attr)).and_then(|_| Ok(prod_attr))
            })
            .map_err(|e: FailureError| {
                e.context(format!("Create new product attribute {:?} error occurred", payload))
                    .into()
            })
    }

    fn update(&self, payload: UpdateProdAttr) -> RepoResult<ProdAttr> {
        debug!("Updating product attribute with payload {:?}.", payload);
        let query = prod_attr_values
            .filter(prod_id.eq(payload.prod_id))
            .filter(attr_id.eq(payload.attr_id));

        query
            .first::<ProdAttr>(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|prod_attr: ProdAttr| acl::check(&*self.acl, Resource::ProductAttrs, Action::Update, self, Some(&prod_attr)))
            .and_then(|_| {
                let filter = prod_attr_values
                    .filter(prod_id.eq(payload.prod_id))
                    .filter(attr_id.eq(payload.attr_id));

                let query = diesel::update(filter).set(&payload);
                query.get_result::<ProdAttr>(self.db_conn).map_err(|e| Error::from(e).into())
            })
            .map_err(|e: FailureError| e.context(format!("Updating product attribute {:?} error occurred", payload)).into())
    }

    /// Finds many product attributes by search terms
    fn find_many(&self, search_terms: ProductAttrsSearchTerms) -> RepoResult<Vec<ProdAttr>> {
        type BoxedExpr = Box<BoxableExpression<prod_attr_values, Pg, SqlType = Bool>>;

        let mut query: BoxedExpr = Box::new(id.eq(id));

        if let Some(attr_id_filter) = search_terms.attr_id {
            query = Box::new(query.and(attr_id.eq(attr_id_filter)));
        }

        if let Some(attr_value_id_filter) = search_terms.attr_value_id {
            query = Box::new(query.and(attr_value_id.eq(attr_value_id_filter)));
        }

        prod_attr_values
            .filter(query)
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|results: Vec<ProdAttr>| {
                for result in results.iter() {
                    acl::check(&*self.acl, Resource::ProductAttrs, Action::Read, self, Some(result))?;
                }
                Ok(results)
            })
            .map_err(|e: FailureError| {
                e.context(format!("Find many product attributes by search terms error occurred"))
                    .into()
            })
    }

    /// Delete all attributes values from product
    fn delete_all_attributes(&self, product_id_arg: ProductId) -> RepoResult<Vec<ProdAttr>> {
        debug!("Delete all attributes of product id {}.", product_id_arg);
        let filtered = prod_attr_values.filter(prod_id.eq(product_id_arg));

        let query = diesel::delete(filtered);
        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|prod_attrs_res: Vec<ProdAttr>| {
                for prod_attr in &prod_attrs_res {
                    acl::check(&*self.acl, Resource::ProductAttrs, Action::Delete, self, Some(&prod_attr))?;
                }
                Ok(prod_attrs_res)
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Delete all attributes values from product by id {:?} error occurred",
                    product_id_arg
                ))
                .into()
            })
    }

    /// Delete all attributes values from product not in the list
    fn delete_all_attributes_not_in_list(&self, product_id_arg: ProductId, attr_values: Vec<i32>) -> RepoResult<Vec<ProdAttr>> {
        debug!(
            "Delete all attributes of product id {} not in the list {:?}.",
            product_id_arg, attr_values
        );
        let filtered = prod_attr_values
            .filter(prod_id.eq(product_id_arg))
            .filter(id.ne_all(attr_values.clone()));

        let query = diesel::delete(filtered);
        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|prod_attrs_res: Vec<ProdAttr>| {
                for prod_attr in &prod_attrs_res {
                    acl::check(&*self.acl, Resource::ProductAttrs, Action::Delete, self, Some(&prod_attr))?;
                }
                Ok(prod_attrs_res)
            })
            .map_err(move |e: FailureError| {
                e.context(format!(
                    "Delete all attributes values not in the list {:?} from product by id {:?} error occurred",
                    attr_values, product_id_arg
                ))
                .into()
            })
    }

    /// Delete attribute value
    fn delete(&self, id_arg: i32) -> RepoResult<ProdAttr> {
        debug!("Delete attribute value by id {}.", id_arg);
        let filtered = prod_attr_values.filter(id.eq(id_arg));

        let query = diesel::delete(filtered);
        query
            .get_result(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|prod_attr: ProdAttr| {
                acl::check(&*self.acl, Resource::ProductAttrs, Action::Delete, self, Some(&prod_attr))?;
                Ok(prod_attr)
            })
            .map_err(|e: FailureError| e.context(format!("Delete attribute value with id {}", id_arg)).into())
    }

    /// Delete attribute values by base_product ID and attribute ID
    fn delete_by_attribute_id(&self, base_product_id: BaseProductId, attribute_id: AttributeId) -> RepoResult<()> {
        debug!(
            "Delete attribute value by base product id {} and attribute id {}.",
            base_product_id, attribute_id
        );

        let filtered = prod_attr_values
            .filter(base_prod_id.eq(base_product_id))
            .filter(attr_id.eq(attribute_id));

        let query = diesel::delete(filtered);
        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|prod_attrs: Vec<ProdAttr>| {
                for prod_attr in prod_attrs {
                    acl::check(&*self.acl, Resource::ProductAttrs, Action::Delete, self, Some(&prod_attr))?;
                }
                Ok(())
            })
            .map_err(|e: FailureError| {
                e.context(format!(
                    "Delete attribute values with base product id {} and attribute id {}",
                    base_product_id, attribute_id
                ))
                .into()
            })
    }

    /// Delete attribute values by base_product ID
    fn delete_by_base_product_id(&self, base_product_id: BaseProductId) -> RepoResult<()> {
        debug!("Delete attribute value by base product id {}.", base_product_id);

        let query = prod_attr_values.filter(base_prod_id.eq(base_product_id));

        query
            .get_results(self.db_conn)
            .map_err(|e| Error::from(e).into())
            .and_then(|prod_attrs: Vec<ProdAttr>| {
                for prod_attr in prod_attrs {
                    acl::check(&*self.acl, Resource::ProductAttrs, Action::Delete, self, Some(&prod_attr))?;
                }
                Ok(())
            })
            .and_then(|_| {
                diesel::delete(query)
                    .execute(self.db_conn)
                    .map_err(|e| Error::from(e).into())
                    .map(|_| ())
            })
            .map_err(|e: FailureError| {
                e.context(format!("Delete attribute values with base product id {}", base_product_id))
                    .into()
            })
    }
}

impl<'a, T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static> CheckScope<Scope, ProdAttr>
    for ProductAttrsRepoImpl<'a, T>
{
    fn is_in_scope(&self, user_id: UserId, scope: &Scope, obj: Option<&ProdAttr>) -> bool {
        match *scope {
            Scope::All => true,
            Scope::Owned => {
                if let Some(prod_attr) = obj {
                    BaseProducts::base_products
                        .filter(BaseProducts::id.eq(prod_attr.base_prod_id))
                        .inner_join(Stores::stores)
                        .get_result::<(BaseProductRaw, Store)>(self.db_conn)
                        .map(|(_, s)| s.user_id == user_id)
                        .ok()
                        .unwrap_or(false)
                } else {
                    false
                }
            }
        }
    }
}
