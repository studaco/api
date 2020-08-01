use chrono::{NaiveTime, Timelike};
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::{
    postgres::{PgPool, PgQueryAs, PgRow},
    Row,
};
use std::vec::Vec;
use thiserror::Error;
use uuid::Uuid;

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, sqlx::Type)]
#[repr(i64)]
pub enum RepetitionFrequency {
    Weekly = 1,
    BiWeekly = 2,
}

#[derive(sqlx::Type)]
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

#[derive(Serialize_repr, Deserialize_repr)]
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

#[derive(Serialize)]
pub struct Time {
    minute: u8,
    hour: u8,
}

#[derive(Serialize)]
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
        let time: NaiveTime = row.try_get("time")?;

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

#[derive(Serialize)]
pub struct Lesson {
    id: Uuid,
    title: String,
    repeats: Vec<Repeat>,
}

impl Repeat {
    async fn for_lesson(db: &PgPool, lesson_id: &Uuid) -> sqlx::Result<Vec<Repeat>> {
        Ok(sqlx::query_as(
            "SELECT every, week_day, scheduled_time FROM Repeats WHERE lesson_id = $1 ",
        )
        .bind(lesson_id)
        .fetch_all(db)
        .await?)
    }
}
