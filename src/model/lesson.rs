use serde::Serialize;
use sqlx::postgres::{ PgPool, PgQueryAs, Postgres };
use std::vec::Vec;
use uuid::Uuid;

use super::permission::{LessonPermission, PermissionType};
use super::repeat::Repeat;
use super::account::AccountID;

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
                let repeats = Repeat::of_lesson_in_transaction(&mut transaction, &lesson_id).await?;

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
        owner: &AccountID,
    ) -> sqlx::Result<Lesson> {
        let mut transaction = db.begin().await?;

        let (id,): (Uuid,) =
            sqlx::query_as("INSERT INTO Lesson (title, description) VALUES ($1,$2) RETURNING id")
                .bind(&title)
                .bind(&description)
                .fetch_one(&mut transaction)
                .await?;

        Repeat::insert_in_transaction(&mut transaction, &repeats, &id).await?;

        LessonPermission::save_in_transaction(
            &mut transaction,
            PermissionType::ReadWrite,
            &id,
            &owner,
        )
        .await?;

        transaction.commit().await?;

        Ok(Lesson {
            id,
            description,
            title,
            repeats,
        })
    }

    pub async fn update(
        db: &PgPool,
        lesson_id: &Uuid,
        title: &Option<String>,
        repeats: &Option<Vec<Repeat>>,
        description: &Option<Option<String>>,
    ) -> sqlx::Result<()> {
        let mut transaction = db.begin().await?;

        if title.is_some() || description.is_some() {
            let setters = match (&title, &description) {
                (Some(_), Some(_)) => "title = $2, description = $3",
                (Some(_), None) => "title = $2",
                (None, Some(_)) => "description = $2",
                (None, None) => "",
            };

            let query_str = format!("UPDATE Lesson SET {} WHERE id = $1", setters);
            let mut query = sqlx::query::<Postgres>(&query_str[..]).bind(lesson_id);
            if let Some(title) = title {
                query = query.bind(title);
            }
            if let Some(description) = description {
                query = query.bind(description);
            }
            query.execute(&mut transaction).await?;
        }

        if let Some(repeats) = repeats {
            Repeat::update_in_transaction(&mut transaction, repeats, lesson_id).await?;
        }

        transaction.commit().await?;

        Ok(())
    }

    pub async fn delete(db: &PgPool, lesson_id: &Uuid) -> sqlx::Result<()> {
        sqlx::query("DELETE FROM Lesson WHERE id = $1")
            .bind(lesson_id)
            .execute(db)
            .await
            .map(|_| ())
    }
}
