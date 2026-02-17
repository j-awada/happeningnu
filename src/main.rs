// Standard library imports

// External crates
use axum::Router;
use axum_messages::MessagesManagerLayer;
use sea_orm::{Database, DatabaseConnection};
use tera::Tera;
use time::Duration as TimeDuration;
use tokio::time::Duration as TokioDuration;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{session_store::ExpiredDeletion, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::{sqlx::SqlitePool, SqliteStore};

// Internal modules
mod entities;
mod handler;
mod router;
mod helper;

// Internal crates
use crate::router::routes;

#[derive(Clone)]
struct AppState {
    db_connection: DatabaseConnection,
    tera: Tera,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url: String = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set as an environment variable.");
    let dbconnection: DatabaseConnection = Database::connect(&database_url).await.unwrap();

    let dbpool = SqlitePool::connect(&database_url).await.unwrap();
    let session_store = SqliteStore::new(dbpool);
    session_store.migrate().await.unwrap();

    let _deletion_task = tokio::task::spawn(
        session_store
        .clone()
        .continuously_delete_expired
        (TokioDuration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(TimeDuration::hours(1)));

    let tera_templates: Tera = Tera::new("templates/*.html").unwrap();

    let app_state: AppState = AppState {
        db_connection: dbconnection,
        tera: tera_templates,
    };

    let assets_dir = ServeDir::new("assets").not_found_service(ServeFile::new("assets/index.html"));

    let app: Router = routes()
        .layer(MessagesManagerLayer)
        .layer(session_layer)
        .nest_service("/assets", assets_dir.clone())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}
