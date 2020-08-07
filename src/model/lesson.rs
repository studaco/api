use chrono::{NaiveTime, Timelike};
use serde::{Serialize, Deserialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sqlx::{
    postgres::{PgPool, PgQueryAs, PgRow},
    Row,
};
use std::vec::Vec;
use thiserror::Error;
use uuid::Uuid;

use crate::model::permission::{LessonPermission, PermissionType};
use crate::model::Transaction;

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, sqlx::Type)]
#[repr(i64)]
pub enum RepetitionFrequency {
    Weekly = 1,
    BiWeekly = 2,
}

#[derive(Copy, Clone, sqlx::Type)]
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

#[derive(Copy, Clone, Serialize_repr, Deserialize_repr)]
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

#[derive(Serialize, Deserialize)]
pub struct Time {
    minute: u8,
    hour: u8,
}

#[derive(Serialize, Deserialize)]
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
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub repeats: Vec<Repeat>,
}

#[derive(sqlx::FromRow)]
struct LessonBase {
    title: String,
    description: Option<String>,
}

impl Repeat {
    pub async fn for_lesson(db: &PgPool, lesson_id: &Uuid) -> sqlx::Result<Vec<Repeat>> {
        Ok(sqlx::query_as(
            "SELECT every, week_day, scheduled_time FROM Repeats WHERE lesson_id = $1 ",
        )
        .bind(lesson_id)
        .fetch_all(db)
        .await?)
    }

    async fn insert_in_transaction(
        transaction: &mut Transaction,
        repeats: &Vec<Repeat>,
        lesson_id: &Uuid,
    ) -> sqlx::Result<()> {
        if !repeats.is_empty() {
            let values = (0..repeats.len())
                .map(|i| {
                    format!(
                        "(${}, ${}, ${}, ${})",
                        i * 4,
                        i * 4 + 1,
                        i * 4 + 2,
                        i * 4 + 3
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
}

impl Lesson {
    pub async fn of_user(db: &PgPool, lesson_id: Uuid) -> sqlx::Result<Option<Lesson>> {
        let mut transaction = db.begin().await?;

        let base = sqlx::query_as("SELECT title, description FROM Lesson WHERE id = $1")
            .bind(&lesson_id)
            .fetch_optional(&mut transaction)
            .await?;

        Ok(match base {
            None => None,
            Some(LessonBase { description, title }) => {
                let repeats: Vec<Repeat> = sqlx::query_as(
                    "SELECT every, week_day, scheduled_time FROM Repeats WHERE lesson_id = $1 ",
                )
                .bind(lesson_id)
                .fetch_all(&mut transaction)
                .await?;

                let res = Lesson {
                    id: lesson_id,
                    title,
                    description,
                    repeats,
                };

                transaction.commit().await?;
                Some(res)
            }
        })
    }

    pub async fn create(
        db: &PgPool,
        title: String,
        description: Option<String>,
        repeats: Vec<Repeat>,
        owner: &Uuid
    ) -> sqlx::Result<Lesson> {
        let mut transaction = db.begin().await?;

        let (id,): (Uuid,) =
            sqlx::query_as("INSERT INTO Lesson (title, description) VALUES ($1, $2) RETURNING id")
                .bind(&title)
                .bind(&description)
                .fetch_one(&mut transaction)
                .await?;

        Repeat::insert_in_transaction(&mut transaction, &repeats, &id).await?;

        LessonPermission::save_in_transaction(&mut transaction, PermissionType::ReadWrite, &id, &owner).await?;

        Ok(Lesson {
            id,
            description,
            title,
            repeats,
        })
    }

    // pub async fn update();
    // pub async fn delete();
}
