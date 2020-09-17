pub mod account;
pub mod permission;
pub mod lesson;
pub mod repeat;
pub mod single_occurrence;
pub mod teacher;

use sqlx::{pool::PoolConnection, postgres::PgConnection};

pub type Transaction = sqlx::Transaction<PoolConnection<PgConnection>>;