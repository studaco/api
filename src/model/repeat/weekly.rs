use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::postgres::PgQueryAs;
use std::vec::Vec;

use crate::model::lesson::LessonID;
use crate::model::{Transaction, templated_insert};

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
pub struct WeeklyRepeat {
    every: i32,
    #[serde(rename = "day")]
    week_day: WeekDay,
    #[serde(rename = "at")]
    scheduled_time: NaiveTime,
    start_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_date: Option<NaiveDate>,
}

impl WeeklyRepeat {
    pub async fn of_lesson_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &LessonID,
    ) -> sqlx::Result<Vec<WeeklyRepeat>> {
        sqlx::query_as("SELECT every, week_day, scheduled_time, start_date, end_date FROM LessonWeeklyRepeat WHERE lesson_id = $1")
            .bind(lesson_id)
            .fetch_all(transaction)
            .await
    }

    pub async fn insert_in_transaction(
        transaction: &mut Transaction,
        repeats: &Vec<WeeklyRepeat>,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        if !repeats.is_empty() {
            let values = (0..repeats.len())
                .map(|i| templated_insert(6, i))
                .collect::<Vec<String>>()
                .join(",");

            let sql = format!(
                "INSERT INTO LessonWeeklyRepeat (every, week_day, scheduled_time, lesson_id, start_date, end_date) VALUES {}",
                values
            );

            let mut query = sqlx::query(&sql[..]);

            for WeeklyRepeat {
                every,
                week_day,
                scheduled_time,
                start_date,
                end_date,
            } in repeats
            {
                query = query
                    .bind(every)
                    .bind(week_day)
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
        repeats: &Vec<WeeklyRepeat>,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        WeeklyRepeat::delete_in_transaction(transaction, lesson_id).await?;
        WeeklyRepeat::insert_in_transaction(transaction, repeats, lesson_id).await
    }

    pub async fn delete_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &LessonID,
    ) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM LessonWeeklyRepeat WHERE lesson_id = $1")
            .bind(lesson_id)
            .execute(transaction)
            .await
            .map(|_| ())
    }

}
