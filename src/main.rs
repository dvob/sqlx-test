use sqlx::FromRow;

#[derive(FromRow, Debug)]
struct Person {
    name: String,
    age: u8,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {

    let pool = sqlx::sqlite::SqlitePoolOptions::new().connect("sqlite://data.db").await?;

    let p = Person {
        name: String::from("John"),
        age: 42,
    };

    let rows: Vec<Person> = sqlx::query_as!(Person, r#"SELECT name, age FROM person;"#)
        .fetch_all(&pool).await?;

    for row in rows {
        println!("{row:?}");
    }
    Ok(())
}