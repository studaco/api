use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryAs;
use std::vec::Vec;

use crate::model::lesson::LessonID;
use crate::model::{Transaction, templated_insert};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, sqlx::FromRow)]
pub struct MonthlyRepeat {
    every: i32,
    #[serde(rename = "at")]
    scheduled_time: NaiveDateTime,
    start_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_date: Option<NaiveDate>,
}

impl MonthlyRepeat {
    pub async fn of_lesson_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &LessonID,
    ) -> sqlx::Result<Vec<MonthlyRepeat>> {
        sqlx::query_as("SELECT every, scheduled_time, start_date, end_date FROM LessonMonthlyRepeat WHERE lesson_id = $1")
            .bind(lesson_id)
            .fetch_all(transaction)
            .await
    }

    pub async fn insert_in_transaction(
        transaction: &mut Transaction,
        repeats: &Vec<MonthlyRepeat>,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        if !repeats.is_empty() {
            let values = (0..repeats.len())
                .map(|i| templated_insert(5, i))
                .collect::<Vec<String>>()
                .join(",");

            let sql = format!(
                "INSERT INTO LessonMonthlyRepeat (every, scheduled_time, lesson_id, start_date, end_date) VALUES {}",
                values
            );

            let mut query = sqlx::query(&sql[..]);

            for MonthlyRepeat {
                every,
                scheduled_time,
                start_date,
                end_date,
            } in repeats
            {
                query = query
                    .bind(every)
                    .bind(scheduled_time)
                    .bind(lesson_id)
                    .bind(start_date)
                    .bind(end_date);
            }
            query.execute(transaction).await?;
        }

        Ok(())
    }

    pub async fn update_in_transaction(
        transaction: &mut Transaction,
        repeats: &Vec<MonthlyRepeat>,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        MonthlyRepeat::delete_in_transaction(transaction, lesson_id).await?;
        MonthlyRepeat::insert_in_transaction(transaction, repeats, lesson_id).await
    }

    pub async fn delete_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM LessonMonthlyRepeat WHERE lesson_id = $1")
            .bind(lesson_id)
            .execute(transaction)
            .await
            .map(|_| ())
    }
}
