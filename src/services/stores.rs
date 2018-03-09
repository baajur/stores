//! Stores Services, presents CRUD operations with stores

use futures_cpupool::CpuPool;
use futures::prelude::*;
use diesel::Connection;
use serde_json;
use stq_acl::UnauthorizedACL;
use stq_static_resources::Translation;
use stq_http::client::ClientHandle;

use models::{NewStore, SearchStore, Store, UpdateStore};
use repos::{StoresRepo, StoresRepoImpl};
use elastic::{StoresSearchRepo, StoresSearchRepoImpl};
use super::types::ServiceFuture;
use super::error::ServiceError as Error;
use repos::types::DbPool;
use repos::acl::{ApplicationAcl, BoxedAcl, RolesCacheImpl};

pub trait StoresService {
    /// Find stores by name limited by `count` parameters
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<Store>>;
    /// Find stores full name by name part limited by `count` parameters
    fn find_full_names_by_name_part(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<String>>;
    /// Returns store by ID
    fn get(&self, store_id: i32) -> ServiceFuture<Store>;
    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> ServiceFuture<Store>;
    /// Creates new store
    fn create(&self, payload: NewStore) -> ServiceFuture<Store>;
    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Store>>;
    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> ServiceFuture<Store>;
}

/// Stores services, responsible for Store-related CRUD operations
pub struct StoresServiceImpl {
    pub db_pool: DbPool,
    pub cpu_pool: CpuPool,
    pub roles_cache: RolesCacheImpl,
    pub user_id: Option<i32>,
    pub client_handle: ClientHandle,
    pub elastic_address: String,
}

impl StoresServiceImpl {
    pub fn new(
        db_pool: DbPool,
        cpu_pool: CpuPool,
        roles_cache: RolesCacheImpl,
        user_id: Option<i32>,
        client_handle: ClientHandle,
        elastic_address: String,
    ) -> Self {
        Self {
            db_pool,
            cpu_pool,
            roles_cache,
            user_id,
            client_handle,
            elastic_address,
        }
    }
}

fn acl_for_id(roles_cache: RolesCacheImpl, user_id: Option<i32>) -> BoxedAcl {
    user_id.map_or(Box::new(UnauthorizedACL::default()) as BoxedAcl, |id| {
        (Box::new(ApplicationAcl::new(roles_cache, id)) as BoxedAcl)
    })
}

impl StoresService for StoresServiceImpl {
    fn find_full_names_by_name_part(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<String>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores_names = {
            let stores_el = StoresSearchRepoImpl::new(client_handle, address);
            let name = search_store.name.clone();
            stores_el
                .find_by_name(search_store, count, offset)
                .map_err(Error::from)
                .and_then(|el_stores| {
                    el_stores
                        .into_iter()
                        .map(move |el_store| {
                            serde_json::from_value::<Vec<Translation>>(el_store.name)
                                .map_err(|e| Error::Parse(e.to_string()))
                                .and_then(|translations| {
                                    translations
                                        .into_iter()
                                        .find(|transl| transl.text.contains(&name))
                                        .ok_or(Error::NotFound)
                                        .map(|t| t.text)
                                })
                        })
                        .collect::<Result<Vec<String>, Error>>()
                        .into_future()
                })
        };

        Box::new(stores_names)
    }

    /// Find stores by name
    fn find_by_name(&self, search_store: SearchStore, count: i64, offset: i64) -> ServiceFuture<Vec<Store>> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let stores = {
            let stores_el = StoresSearchRepoImpl::new(client_handle, address);
            stores_el
                .find_by_name(search_store, count, offset)
                .map_err(Error::from)
        };

        Box::new(stores.and_then({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;
            let roles_cache = self.roles_cache.clone();
            move |el_stores| {
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            el_stores
                                .into_iter()
                                .map(|el_store| {
                                    let acl = acl_for_id(roles_cache.clone(), user_id);
                                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                                    stores_repo.find(el_store.id).map_err(Error::from)
                                })
                                .collect()
                        })
                })
            }
        }))
    }

    /// Returns store by ID
    fn get(&self, store_id: i32) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);

                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.find(store_id).map_err(Error::from)
                })
        }))
    }

    /// Deactivates specific store
    fn deactivate(&self, store_id: i32) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.deactivate(store_id).map_err(Error::from)
                })
        }))
    }

    /// Lists users limited by `from` and `count` parameters
    fn list(&self, from: i32, count: i64) -> ServiceFuture<Vec<Store>> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);
                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo.list(from, count).map_err(Error::from)
                })
        }))
    }

    /// Creates new store
    fn create(&self, payload: NewStore) -> ServiceFuture<Store> {
        let client_handle = self.client_handle.clone();
        let address = self.elastic_address.clone();
        let check_store_name_exists = {
            serde_json::from_value::<Vec<Translation>>(payload.name.clone())
                .map_err(|e| Error::Parse(e.to_string()))
                .into_future()
                .and_then(|translations| {
                    let stores_el = StoresSearchRepoImpl::new(client_handle, address);
                    stores_el
                        .name_exists(translations)
                        .map(move |exists| (payload, exists))
                        .map_err(Error::from)
                        .and_then(|(payload, exists)| {
                            if exists {
                                Err(Error::Validate(
                                    validation_errors!({"name": ["name" => "Store with this name already exists"]}),
                                ))
                            } else {
                                Ok(payload)
                            }
                        })
                })
        };

        Box::new(check_store_name_exists.and_then({
            let cpu_pool = self.cpu_pool.clone();
            let db_pool = self.db_pool.clone();
            let user_id = self.user_id;
            let roles_cache = self.roles_cache.clone();
            move |new_store| {
                cpu_pool.spawn_fn(move || {
                    db_pool
                        .get()
                        .map_err(|e| Error::Connection(e.into()))
                        .and_then(move |conn| {
                            let acl = acl_for_id(roles_cache, user_id);
                            let stores_repo = StoresRepoImpl::new(&conn, acl);
                            conn.transaction::<Store, Error, _>(move || {
                                stores_repo
                                    .slug_exists(new_store.slug.to_string())
                                    .map(move |exists| (new_store, exists))
                                    .map_err(Error::from)
                                    .and_then(|(new_store, exists)| match exists {
                                        false => Ok(new_store),
                                        true => Err(Error::Validate(
                                            validation_errors!({"slug": ["slug" => "Slug already exists"]}),
                                        )),
                                    })
                                    .and_then(move |new_store| stores_repo.create(new_store).map_err(Error::from))
                            })
                        })
                })
            }
        }))
    }

    /// Updates specific store
    fn update(&self, store_id: i32, payload: UpdateStore) -> ServiceFuture<Store> {
        let db_pool = self.db_pool.clone();
        let user_id = self.user_id;
        let roles_cache = self.roles_cache.clone();

        Box::new(self.cpu_pool.spawn_fn(move || {
            db_pool
                .get()
                .map_err(|e| Error::Connection(e.into()))
                .and_then(move |conn| {
                    let acl = acl_for_id(roles_cache, user_id);

                    let stores_repo = StoresRepoImpl::new(&conn, acl);
                    stores_repo
                        .find(store_id.clone())
                        .and_then(move |_user| stores_repo.update(store_id, payload))
                        .map_err(Error::from)
                })
        }))
    }
}
