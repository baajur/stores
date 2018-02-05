use futures::future::Future;
use super::error::Error;
use diesel::pg::PgConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;

/// Repos layer Future
pub type RepoFuture<T> = Box<Future<Item = T, Error = Error>>;
pub type RepoResult<T> = Result<T, Error>;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;
