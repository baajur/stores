//! Stores Services, presents CRUD operations with stores
use std::collections::HashMap;

use diesel::connection::AnsiTransactionManager;
use diesel::pg::Pg;
use diesel::Connection;
use futures::prelude::*;
use futures_cpupool::CpuPool;
use r2d2::{ManageConnection, Pool};

use serde_json;
use stq_http::client::ClientHandle;
use stq_static_resources::Translation;

use super::error::ServiceError;
use super::types::ServiceFuture;
use elastic::{StoresElastic, StoresElasticImpl};
use models::{BaseProductWithVariants, CartProduct, Category, NewStore, Product, SearchStore, Store, StoreWithBaseProducts, UpdateStore};
use repos::remove_unused_categories;
use repos::{RepoResult, ReposFactory};

pub trait StoresService {
    /// Find stores by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i32, offset: i32) -> ServiceFuture<Vec<Store>>;
    /// search filters count
    fn search_filters_count(&self, search_store: SearchStore) -> ServiceFuture<i32>;
    /// search filters country
    fn search_filters_country(&self, search_store: SearchStore) -> ServiceFuture<Vec<String>>;
    /// search filters category
    fn search_filters_category(&self, search_store: SearchStore) -> ServiceFuture<Category>;
    /// Find stores auto complete limited by `count` parameters
    fn auto_complete(&self, name: String, count: i32, offset: i32) -> ServiceFuture<Vec<String>>;
    /// Returns store by ID
    fn get(&self, store_id: i32) -> ServiceFuture<Store>;
    /// Returns products count
    fn get_products_count(&self, store_id: i32) -> ServiceFuture<i32>;
    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> ServiceFuture<Store>;
    /// Creates new store
    fn create(&self, payload: NewStore) -> ServiceFuture<Store>;
    /// Lists stores limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<Store>>;
    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> ServiceFuture<Store>;
    /// Cart
    fn find_by_cart(&self, cart: Vec<CartProduct>) -> ServiceFuture<Vec<StoreWithBaseProducts>>;
}

/// Stores services, responsible for Store-related CRUD operations
pub struct StoresServiceImpl<
    T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
    M: ManageConnection<Connection = T>,
    F: ReposFactory<T>,
> {
    pub db_pool: Pool<M>,
    pub cpu_pool: CpuPool,
    pub user_id: Option<i32>,
    pub client_handle: ClientHandle,
    pub elastic_address: String,
    pub repo_factory: F,
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > StoresServiceImpl<T, M, F>
{
    pub fn new(
        db_pool: Pool<M>,
        cpu_pool: CpuPool,
        user_id: Option<i32>,
        client_handle: ClientHandle,
        elastic_address: String,
        repo_factory: F,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            user_id,
            client_handle,
            elastic_address,
            repo_factory,
        }
    }
}

impl<
        T: Connection<Backend = Pg, TransactionManager = AnsiTransactionManager> + 'static,
        M: ManageConnection<Connection = T>,
        F: ReposFactory<T>,
    > StoresService for StoresServiceImpl<T, M, F>
{
    fn auto_complete(&self, name: String, count: i32, offset: i32) -> ServiceFuture<Vec<String>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores_names = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el.auto_complete(name, count, offset).map_err(ServiceError::from)
        };

        Box::new(stores_names)
    }

    /// Find stores by name
    fn find_by_name(&self, search_store: SearchStore, count: i32, offset: i32) -> ServiceFuture<Vec<Store>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el.find_by_name(search_store, count, offset).map_err(ServiceError::from)
        };

        Box::new(stores.and_then({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;

            let repo_factory = self.repo_factory.clone();
            move |el_stores| {
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| {
                            error!("Could not get connection to db from pool! {}", e.to_string());
                            ServiceError::Connection(e.into())
                        })
                        .and_then(move |conn| {
                            el_stores
                                .into_iter()
                                .map(|el_store| {
                                    let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                                    stores_repo.find(el_store.id).map_err(ServiceError::from)
                                })
                                .collect()
                        })
                })
            }
        }))
    }

    /// search filters count
    fn search_filters_count(&self, search_store: SearchStore) -> ServiceFuture<i32> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let search_filters = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el.search_count(search_store).map_err(ServiceError::from)
        };

        Box::new(search_filters)
    }

    /// search filters country
    fn search_filters_country(&self, search_store: SearchStore) -> ServiceFuture<Vec<String>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let search_filters = {
            let stores_el = StoresElasticImpl::new(client_handle, address);
            stores_el.aggregate_countries(search_store).map_err(ServiceError::from)
        };

        Box::new(search_filters)
    }

    /// search filters category
    fn search_filters_category(&self, search_store: SearchStore) -> ServiceFuture<Category> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores_el = StoresElasticImpl::new(client_handle, address);
        let cpu_pool = self.cpu_pool.clone();
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let repo_factory = self.repo_factory.clone();

        Box::new(
            stores_el
                .aggregate_categories(search_store)
                .map_err(ServiceError::from)
                .and_then(move |categories_ids| {
                    cpu_pool.spawn_fn(move || {
                        db_pool
                            .get()
                            .map_err(|e| {
                                error!("Could not get connection to db from pool! {}", e.to_string());
                                ServiceError::Connection(e.into())
                            })
                            .and_then(move |conn| {
                                let categories_repo = repo_factory.create_categories_repo(&*conn, user_id);
                                categories_repo.get_all().map_err(ServiceError::from)
                            })
                            .and_then(|category| {
                                let new_cat = remove_unused_categories(category, &categories_ids, 0);
                                Ok(new_cat)
                            })
                    })
                }),
        )
    }

    /// Returns store by ID
    fn get(&self, store_id: i32) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!("Could not get connection to db from pool! {}", e.to_string());
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                    stores_repo.find(store_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Returns products count
    fn get_products_count(&self, store_id: i32) -> ServiceFuture<i32> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!("Could not get connection to db from pool! {}", e.to_string());
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                    base_products_repo.count_with_store_id(store_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!("Could not get connection to db from pool! {}", e.to_string());
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                    stores_repo.deactivate(store_id).map_err(ServiceError::from)
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i32) -> ServiceFuture<Vec<Store>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!("Could not get connection to db from pool! {}", e.to_string());
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                    stores_repo.list(from, count).map_err(ServiceError::from)
                })
        }))
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> ServiceFuture<Store> {
        Box::new({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;

            let repo_factory = self.repo_factory.clone();
            cpu_pool.spawn_fn(move || {
                db_pool
                    .get()
                    .map_err(|e| {
                        error!("Could not get connection to db from pool! {}", e.to_string());
                        ServiceError::Connection(e.into())
                    })
                    .and_then(move |conn| {
                        let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                        conn.transaction::<Store, ServiceError, _>(move || {
                            serde_json::from_value::<Vec<Translation>>(payload.name.clone())
                                .map_err(|e| ServiceError::Parse(e.to_string()))
                                .and_then(|translations| {
                                    stores_repo
                                        .name_exists(translations)
                                        .map(move |exists| (payload, exists))
                                        .map_err(ServiceError::from)
                                        .and_then(|(payload, exists)| {
                                            if exists {
                                                Err(ServiceError::Validate(
                                                    validation_errors!({"name": ["name" => "Store with this name already exists"]}),
                                                ))
                                            } else {
                                                Ok(payload)
                                            }
                                        })
                                })
                                .and_then(|payload| {
                                    stores_repo
                                        .slug_exists(payload.slug.to_string())
                                        .map(move |exists| (payload, exists))
                                        .map_err(ServiceError::from)
                                        .and_then(|(new_store, exists)| {
                                            if exists {
                                                Err(ServiceError::Validate(
                                                    validation_errors!({"slug": ["slug" => "Store with this slug already exists"]}),
                                                ))
                                            } else {
                                                Ok(new_store)
                                            }
                                        })
                                })
                                .and_then(move |new_store| stores_repo.create(new_store).map_err(ServiceError::from))
                        })
                    })
            })
        })
    }

    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!("Could not get connection to db from pool! {}", e.to_string());
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                    stores_repo
                        .find(store_id)
                        .map_err(ServiceError::from)
                        .and_then(|s| {
                            if let Some(slug) = payload.slug.clone() {
                                if s.slug == slug {
                                    // if updated slug equal store slug
                                    Ok(false)
                                } else {
                                    // if updated slug equal other stores slug
                                    stores_repo.slug_exists(slug).map_err(ServiceError::from)
                                }
                            } else {
                                Ok(false)
                            }
                        })
                        .and_then(|exists| {
                            if exists {
                                Err(ServiceError::Validate(
                                    validation_errors!({"slug": ["slug" => "Store with this slug already exists"]}),
                                ))
                            } else {
                                Ok(())
                            }
                        })
                        .and_then(move |_| stores_repo.update(store_id, payload).map_err(ServiceError::from))
                })
        }))
    }

    /// Find by cart
    fn find_by_cart(&self, cart: Vec<CartProduct>) -> ServiceFuture<Vec<StoreWithBaseProducts>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;

        let repo_factory = self.repo_factory.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| {
                    error!("Could not get connection to db from pool! {}", e.to_string());
                    ServiceError::Connection(e.into())
                })
                .and_then(move |conn| {
                    let stores_repo = repo_factory.create_stores_repo(&*conn, user_id);
                    let base_products_repo = repo_factory.create_base_product_repo(&*conn, user_id);
                    let products_repo = repo_factory.create_product_repo(&*conn, user_id);
                    let products = cart.into_iter()
                        .map(|cart_product| products_repo.find(cart_product.product_id))
                        .collect::<RepoResult<Vec<Product>>>();
                    products
                        .and_then(|products| {
                            let mut group_by_base_product_id = HashMap::<i32, Vec<Product>>::default();
                            for product in products {
                                let p = group_by_base_product_id.entry(product.base_product_id).or_insert(vec![]);
                                p.push(product);
                            }
                            group_by_base_product_id
                                .into_iter()
                                .map(|(base_product_id, products)| {
                                    base_products_repo
                                        .find(base_product_id)
                                        .map(|base_product| BaseProductWithVariants::new(base_product, products))
                                })
                                .collect::<RepoResult<Vec<BaseProductWithVariants>>>()
                        })
                        .and_then(|base_products| {
                            let mut group_by_store_id = HashMap::<i32, Vec<BaseProductWithVariants>>::default();
                            for base_product in base_products {
                                let bp = group_by_store_id.entry(base_product.store_id).or_insert(vec![]);
                                bp.push(base_product);
                            }
                            group_by_store_id
                                .into_iter()
                                .map(|(store_id, base_products)| {
                                    stores_repo
                                        .find(store_id)
                                        .map(|store| StoreWithBaseProducts::new(store, base_products))
                                })
                                .collect::<RepoResult<Vec<StoreWithBaseProducts>>>()
                        })
                        .map_err(ServiceError::from)
                })
        }))
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use futures_cpupool::CpuPool;
    use r2d2;
    use serde_json;
    use tokio_core::reactor::Core;
    use tokio_core::reactor::Handle;

    use stq_http;
    use stq_http::client::Config as HttpConfig;

    use config::Config;
    use models::*;
    use repos::repo_factory::tests::*;
    use services::*;

    fn create_store_service(
        user_id: Option<i32>,
        handle: Arc<Handle>,
    ) -> StoresServiceImpl<MockConnection, MockConnectionManager, ReposFactoryMock> {
        let manager = MockConnectionManager::default();
        let db_pool = r2d2::Pool::builder().build(manager).expect("Failed to create connection pool");
        let cpu_pool = CpuPool::new(1);

        let config = Config::new().unwrap();
        let http_config = HttpConfig {
            http_client_retries: config.client.http_client_retries,
            http_client_buffer_size: config.client.http_client_buffer_size,
        };
        let client = stq_http::client::Client::new(&http_config, &handle);
        let client_handle = client.handle();

        StoresServiceImpl {
            db_pool: db_pool,
            cpu_pool: cpu_pool,
            user_id: user_id,
            elastic_address: "127.0.0.1:9200".to_string(),
            client_handle: client_handle,
            repo_factory: MOCK_REPO_FACTORY,
        }
    }

    pub fn create_new_store(name: serde_json::Value) -> NewStore {
        NewStore {
            name: name,
            user_id: MOCK_USER_ID,
            short_description: serde_json::from_str("{}").unwrap(),
            long_description: None,
            slug: "slug".to_string(),
            cover: None,
            logo: None,
            phone: Some("1234567".to_string()),
            email: Some("example@mail.com".to_string()),
            address: Some("town city street".to_string()),
            facebook_url: None,
            twitter_url: None,
            instagram_url: None,
            default_language: "en".to_string(),
            slogan: Some("fdsf".to_string()),
            country: None,
            administrative_area_level_1: None,
            administrative_area_level_2: None,
            locality: None,
            political: None,
            postal_code: None,
            route: None,
            street_number: None,
        }
    }

    pub fn create_update_store(name: serde_json::Value) -> UpdateStore {
        UpdateStore {
            name: Some(name),
            short_description: serde_json::from_str("{}").unwrap(),
            long_description: None,
            slug: None,
            cover: None,
            logo: None,
            phone: None,
            email: None,
            address: None,
            facebook_url: None,
            twitter_url: None,
            instagram_url: None,
            default_language: None,
            slogan: None,
            rating: None,
            country: None,
            product_categories: None,
            status: None,
            administrative_area_level_1: None,
            administrative_area_level_2: None,
            locality: None,
            political: None,
            postal_code: None,
            route: None,
            street_number: None,
        }
    }

    #[test]
    fn test_get_store() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_store_service(Some(MOCK_USER_ID), handle);
        let work = service.get(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
    }

    #[test]
    fn test_create_allready_existed() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_store_service(Some(MOCK_USER_ID), handle);
        let new_store = create_new_store(serde_json::from_str(MOCK_STORE_NAME_JSON_EXISTED).unwrap());
        let work = service.create(new_store);
        let result = core.run(work);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn test_list() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_store_service(Some(MOCK_USER_ID), handle);
        let work = service.list(1, 5);
        let result = core.run(work).unwrap();
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_create_store() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_store_service(Some(MOCK_USER_ID), handle);
        let new_store = create_new_store(serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
        let work = service.create(new_store);
        let result = core.run(work).unwrap();
        assert_eq!(
            result.name,
            serde_json::from_str::<serde_json::Value>(MOCK_STORE_NAME_JSON).unwrap()
        );
    }

    #[test]
    fn test_update() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_store_service(Some(MOCK_USER_ID), handle);
        let new_store = create_update_store(serde_json::from_str(MOCK_STORE_NAME_JSON).unwrap());
        let work = service.update(1, new_store);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(
            result.name,
            serde_json::from_str::<serde_json::Value>(MOCK_STORE_NAME_JSON).unwrap()
        );
    }

    #[test]
    fn test_deactivate() {
        let mut core = Core::new().unwrap();
        let handle = Arc::new(core.handle());
        let service = create_store_service(Some(MOCK_USER_ID), handle);
        let work = service.deactivate(1);
        let result = core.run(work).unwrap();
        assert_eq!(result.id, 1);
        assert_eq!(result.is_active, false);
    }

}
