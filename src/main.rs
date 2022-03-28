use std::{time::Duration, sync::Arc, net::SocketAddr};

use axum::{extract::{Extension, Path}, Router, routing::get, error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse, Json};
use clap::{Parser, Subcommand};

use serde::Deserialize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::TraceLayer;

use tower::{BoxError, ServiceBuilder};
use user::{SQLiteUserStore, User};
use uuid::Uuid;

mod user;

#[derive(Parser)]
struct RootCommand {
    #[clap(short, long, default_value = "sqlite://data.db")]
    connect_string: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Get { id: Option<uuid::Uuid> },
    Create { name: String, age: u8 },
    Server,
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
        Command::Server => {
            server(store).await;
        }
    }
    Ok(())
}

async fn server(store: SQLiteUserStore) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "example_todos=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Compose the routes
    let app = Router::new()
        .route("/user", get(user_list).post(user_create))
        .route("/user/:id", get(user_get).delete(user_delete))
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .layer(Extension(Arc::new(store)))
                .into_inner(),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn user_list(
    Extension(store): Extension<Arc<SQLiteUserStore>>,
) -> impl IntoResponse {
    let users = store.get_users().await.unwrap();
    Json(users)
}

#[derive(Deserialize)]
struct NewUser {
    name: String,
    age: u8,
}

async fn user_create(
    Json(user): Json<NewUser>,
    Extension(store): Extension<Arc<SQLiteUserStore>>,
) -> impl IntoResponse {
    let user = User::new(user.name, user.age);

    store.create_user(user.clone()).await.unwrap();

    (StatusCode::CREATED, Json(user))
}

async fn user_get(
    Path(id): Path<Uuid>,
    Extension(store): Extension<Arc<SQLiteUserStore>>,
) -> impl IntoResponse {

    let user = store.get_user_by_id(id).await.unwrap();
    Json(user)
}

async fn user_delete(
    Path(id): Path<Uuid>,
    Extension(store): Extension<Arc<SQLiteUserStore>>,
) -> impl IntoResponse {

    let user = store.delete_user(id).await.unwrap();
}