use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryAs;
use std::vec::Vec;

use super::lesson::LessonID;
use super::Transaction;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, sqlx::Type)]
#[sqlx(transparent)]
pub struct SingleOccurance(NaiveDateTime);

impl SingleOccurance {
    pub async fn of_lesson_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &LessonID,
    ) -> sqlx::Result<Vec<SingleOccurance>> {
        sqlx::query_as::<_, (SingleOccurance,)>(
            "SELECT occures_at FROM SingleOccurance WHERE lesson_id = $1",
        )
        .bind(lesson_id)
        .fetch_all(transaction)
        .await
        .map(|vec| vec.into_iter().map(|(occurance,)| occurance).collect())
    }

    pub async fn insert_in_transaction(
        transaction: &mut Transaction,
        singles: &Vec<SingleOccurance>,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        if !singles.is_empty() {
            let values = (0..singles.len())
                .map(|i| format!("(${}, ${})", i * 2 + 1, i * 2 + 2,))
                .collect::<Vec<String>>()
                .join(",");

            let sql = format!(
                "INSERT INTO SingleOccurance (occures_at, lesson_id) VALUES {}",
                values
            );

            let mut query = sqlx::query(&sql[..]);

            for SingleOccurance(occurance) in singles {
                query = query.bind(occurance).bind(lesson_id);
            }
            query.execute(transaction).await?;
        }

        Ok(())
    }

    pub async fn update_in_transaction(
        transaction: &mut Transaction,
        singles: &Vec<SingleOccurance>,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        SingleOccurance::delete_in_transaction(transaction, lesson_id).await?;
        SingleOccurance::insert_in_transaction(transaction, singles, lesson_id).await
    }

    pub async fn delete_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM SingleOccurance WHERE lesson_id = $1")
            .bind(lesson_id)
            .execute(transaction)
            .await
            .map(|_| ())
    }
}
