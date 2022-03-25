use rand::Rng;

mod user;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //  for SQLite, use SqlitePoolOptions::new()

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite://data.db")
        .await?;

    let mut rng = rand::thread_rng();

    let user = user::User::new(String::from("rand person1"), rng.gen());

    let store = user::SQLiteUserStore::new(pool);

    store.create_user(user).await?;

    let users = store.get_users().await?;

    for user in users {
        println!("{user:?}");
    }
    Ok(())
}
