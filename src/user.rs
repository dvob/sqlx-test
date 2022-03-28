use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

#[derive(FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
    pub age: u8,
}

impl User {
    pub fn new(name: String, age: u8) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name,
            age,
        }
    }
}

pub struct SQLiteUserStore {
    pool: Pool<Sqlite>,
}

impl SQLiteUserStore {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: User) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "INSERT INTO user (id, name, age) VALUES (?, ?, ?);",
            user.id,
            user.name,
            user.age
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_users(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
        // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL)
        Ok(sqlx::query_as!(
            User,
            r#"SELECT id as "id: uuid::Uuid", name, age as "age: u8" FROM user;"#
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn get_user_by_id(&self, id: uuid::Uuid) -> Result<User, Box<dyn std::error::Error>> {
        // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL)
        Ok(sqlx::query_as!(
            User,
            r#"SELECT id as "id: uuid::Uuid", name, age as "age: u8" FROM user WHERE id = ?;"#,
            id,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn delete_user(&self, id: uuid::Uuid) -> Result<(), Box<dyn std::error::Error>> {
        // Make a simple query to return the given parameter (use a question mark `?` instead of `$1` for MySQL)
        sqlx::query!(
            r#"DELETE FROM user WHERE id = ?;"#,
            id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
