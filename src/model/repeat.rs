use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::postgres::PgQueryAs;
use std::vec::Vec;

use super::lesson::LessonID;
use super::Transaction;

#[derive(Debug, Copy, Clone, Serialize_repr, Deserialize_repr, sqlx::Type)]
#[repr(i16)]
pub enum WeekDay {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, sqlx::FromRow)]
pub struct Repeat {
    every: i32,
    #[serde(rename = "day")]
    week_day: WeekDay,
    #[serde(rename = "time")]
    scheduled_time: NaiveTime,
    start_day: NaiveDate,
    end_day: Option<NaiveDate>,
}

impl Repeat {
    pub async fn of_lesson_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &LessonID,
    ) -> sqlx::Result<Vec<Repeat>> {
        sqlx::query_as("SELECT every, week_day, scheduled_time, start_day, end_day FROM Repeats WHERE lesson_id = $1")
            .bind(lesson_id)
            .fetch_all(transaction)
            .await
    }

    pub async fn insert_in_transaction(
        transaction: &mut Transaction,
        repeats: &Vec<Repeat>,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        if !repeats.is_empty() {
            let values = (0..repeats.len())
                .map(|i| {
                    format!(
                        "(${}, ${}, ${}, ${}, ${}, ${})",
                        i * 6 + 1,
                        i * 6 + 2,
                        i * 6 + 3,
                        i * 6 + 4,
                        i * 6 + 5,
                        i * 6 + 6,
                    )
                })
                .collect::<Vec<String>>()
                .join(",");

            let sql = format!(
                "INSERT INTO Repeats (every, week_day, scheduled_time, lesson_id, start_day, end_day) VALUES {}",
                values
            );

            let mut query = sqlx::query(&sql[..]);

            for Repeat {
                every,
                week_day,
                scheduled_time,
                start_day,
                end_day,
            } in repeats
            {
                query = query
                    .bind(every)
                    .bind(week_day)
                    .bind(scheduled_time)
                    .bind(lesson_id)
                    .bind(start_day)
                    .bind(end_day);
            }
            query.execute(transaction).await?;
        }

        Ok(())
    }

    pub async fn update_in_transaction(
        transaction: &mut Transaction,
        repeats: &Vec<Repeat>,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        Repeat::delete_in_transaction(transaction, lesson_id).await?;
        Repeat::insert_in_transaction(transaction, repeats, lesson_id).await
    }

    pub async fn delete_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM Repeats WHERE lesson_id = $1")
            .bind(lesson_id)
            .execute(transaction)
            .await
            .map(|_| ())
    }

}
