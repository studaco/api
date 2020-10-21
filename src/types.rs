use sqlx::{pool::PoolConnection, postgres::PgConnection};

pub type Transaction = sqlx::Transaction<PoolConnection<PgConnection>>;
pub type RedisPool = deadpool_redis::Pool;