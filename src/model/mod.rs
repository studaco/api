pub mod account;
pub mod permission;
pub mod lesson;

use sqlx::{pool::PoolConnection, postgres::PgConnection};

pub type Transaction = sqlx::Transaction<PoolConnection<PgConnection>>;