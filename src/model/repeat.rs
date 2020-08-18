use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::{
    postgres::{PgQueryAs, PgRow},
    Row,
};
use std::vec::Vec;
use thiserror::Error;

use super::lesson::LessonID;
use super::Transaction;

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(rename = "weekday")]
enum PgWeekDay {
    MON,
    TUE,
    WED,
    THU,
    FRI,
    SAT,
    SUN,
}

impl From<WeekDay> for PgWeekDay {
    fn from(week_day: WeekDay) -> Self {
        match week_day {
            WeekDay::Monday => PgWeekDay::MON,
            WeekDay::Tuesday => PgWeekDay::TUE,
            WeekDay::Wednesday => PgWeekDay::WED,
            WeekDay::Thursday => PgWeekDay::THU,
            WeekDay::Friday => PgWeekDay::FRI,
            WeekDay::Saturday => PgWeekDay::SAT,
            WeekDay::Sunday => PgWeekDay::SUN,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WeekDay {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl From<PgWeekDay> for WeekDay {
    fn from(week_day: PgWeekDay) -> Self {
        match week_day {
            PgWeekDay::MON => WeekDay::Monday,
            PgWeekDay::TUE => WeekDay::Tuesday,
            PgWeekDay::WED => WeekDay::Wednesday,
            PgWeekDay::THU => WeekDay::Thursday,
            PgWeekDay::FRI => WeekDay::Friday,
            PgWeekDay::SAT => WeekDay::Saturday,
            PgWeekDay::SUN => WeekDay::Sunday,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Repeat {
    every: i32,
    day: WeekDay,
    time: NaiveTime,
    start_day: NaiveDate,
    end_day: Option<NaiveDate>,
}

#[derive(Debug, Error)]
#[error("Wrong repetition frequency")]
struct WrongRepetitionFrequency {}

impl<'c> sqlx::FromRow<'c, PgRow<'c>> for Repeat {
    fn from_row(row: &PgRow<'c>) -> sqlx::Result<Self> {
        let every = row.try_get("every")?;
        let day: PgWeekDay = row.try_get("week_day")?;
        let time = row.try_get("scheduled_time")?;
        let start_day = row.try_get("start_day")?;
        let end_day = row.try_get("end_day")?;

        Ok(Repeat {
            every,
            day: day.into(),
            time,
            start_day,
            end_day,
        })
    }
}

impl Repeat {
    pub async fn of_lesson_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &LessonID,
    ) -> sqlx::Result<Vec<Repeat>> {
        sqlx::query_as("SELECT every, week_day, scheduled_time FROM Repeats WHERE lesson_id = $1")
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
                .map(|_| "(?, ?, ?, ?, ?, ?)")
                .collect::<Vec<&'static str>>()
                .join(",");

            let sql = format!(
                "INSERT INTO Repeats (every, week_day, scheduled_time, lesson_id, start_day, end_day) VALUES {}",
                values
            );

            let mut query = sqlx::query(&sql[..]);

            for Repeat {
                every,
                day,
                time,
                start_day,
                end_day,
            } in repeats
            {
                let week_day: PgWeekDay = day.clone().into();
                query = query
                    .bind(every)
                    .bind(week_day)
                    .bind(time)
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
