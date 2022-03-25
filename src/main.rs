use clap::{Parser, Subcommand};

mod user;

#[derive(Parser)]
struct RootCommand {
    #[clap(short, long, default_value = "sqlite://data.db")]
    connect_string: String,

    #[clap(subcommand)]
    command: Command
}

#[derive(Subcommand)]
enum Command {
    Get {
        id: Option<uuid::Uuid>,
    },
    Create {
        name: String,
        age: u8,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //  for SQLite, use SqlitePoolOptions::new()
    let cmd = RootCommand::parse();


    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(cmd.connect_string.as_str())
        .await?;

    let store = user::SQLiteUserStore::new(pool);

    match cmd.command {
        Command::Create { name, age } => {
            let user = user::User::new(name, age);
            store.create_user(user).await?;
        }
        Command::Get { id } => match id {
            Some(id) => {
                let user = store.get_user_by_id(id).await?;
                println!("{user:?}");
            }
            None => {
                let users = store.get_users().await?;
                for user in users {
                    println!("{user:?}")
                }
            }
        },
    }
    Ok(())
}
