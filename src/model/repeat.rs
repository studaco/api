use chrono::{NaiveTime, Timelike};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::{
    postgres::{PgPool, PgQueryAs, PgRow},
    Row,
};
use std::vec::Vec;
use thiserror::Error;
use uuid::Uuid;

use super::Transaction;

#[derive(Debug, Serialize_repr, Deserialize_repr, Copy, Clone, sqlx::Type)]
#[repr(i32)]
pub enum RepetitionFrequency {
    Weekly = 1,
    BiWeekly = 2,
}

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
pub struct Time {
    minute: u8,
    hour: u8,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Repeat {
    every: RepetitionFrequency,
    day: WeekDay,
    time: Time,
}

#[derive(Debug, Error)]
#[error("Wrong repetition frequency")]
struct WrongRepetitionFrequency {}

impl<'c> sqlx::FromRow<'c, PgRow<'c>> for Repeat {
    fn from_row(row: &PgRow<'c>) -> sqlx::Result<Self> {
        let every: RepetitionFrequency = row.try_get("every")?;
        let day: PgWeekDay = row.try_get("week_day")?;
        let time: NaiveTime = row.try_get("scheduled_time")?;

        Ok(Repeat {
            every,
            day: day.into(),
            time: Time {
                minute: time.minute() as u8,
                hour: time.hour() as u8,
            },
        })
    }
}

impl Repeat {
    pub async fn of_lesson_in_transaction(transaction: &mut Transaction, lesson_id: &Uuid) -> sqlx::Result<Vec<Repeat>> {
        sqlx::query_as(
            "SELECT every, week_day, scheduled_time FROM Repeats WHERE lesson_id = $1 ",
        )
        .bind(lesson_id)
        .fetch_all(transaction)
        .await
    }

    pub async fn insert_in_transaction(
        transaction: &mut Transaction,
        repeats: &Vec<Repeat>,
        lesson_id: &Uuid,
    ) -> sqlx::Result<()> {
        if !repeats.is_empty() {
            let values = (0..repeats.len())
                .map(|i| {
                    format!(
                        "(${}, ${}, ${}, ${})",
                        i * 4 + 1,
                        i * 4 + 2,
                        i * 4 + 3,
                        i * 4 + 4
                    )
                    .to_string()
                })
                .collect::<Vec<String>>()
                .join(",");

            let sql = format!(
                "INSERT INTO Repeats (every, week_day, scheduled_time, lesson_id) VALUES {}",
                values
            );

            let mut query = sqlx::query(&sql[..]);

            for Repeat {
                every,
                day,
                time: Time { hour, minute },
            } in repeats
            {
                let time = NaiveTime::from_hms(hour.clone() as u32, minute.clone() as u32, 0);
                let week_day: PgWeekDay = day.clone().into();
                query = query.bind(every).bind(week_day).bind(time).bind(lesson_id);
            }
            query.execute(transaction).await?;
        }

        Ok(())
    }

    pub async fn update_in_transaction(
        transaction: &mut Transaction,
        repeats: &Vec<Repeat>,
        lesson_id: &Uuid,
    ) -> sqlx::Result<()> {
        Repeat::delete_in_transaction(transaction, lesson_id).await?;
        Repeat::insert_in_transaction(transaction, repeats, lesson_id).await
    }

    pub async fn delete_in_transaction(
        transaction: &mut Transaction,
        lesson_id: &Uuid,
    ) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM Repeats WHERE lesson_id = $1")
            .bind(lesson_id)
            .execute(transaction)
            .await
            .map(|_| ())
    }
}