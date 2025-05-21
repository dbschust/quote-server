mod error;
mod quote;
mod templates;

use error::*;
use quote::*;
use templates::*;

extern crate mime;

use axum::{self, extract::State, response, routing};
use clap::Parser;
extern crate fastrand;
use sqlx::{SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};
use tower_http::{services, /*trace*/};
//use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::Arc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
}
struct AppState {
    db: SqlitePool,
}

async fn get_quote(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let app_state = app_state.read().await;
    let db = &app_state.db;
    let quote = sqlx::query_as!(Quote, "SELECT * FROM quotes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await
        .unwrap();
    let quote = IndexTemplate::quote(&quote);
    response::Html(quote.to_string())
}

fn get_db_uri(db_uri: Option<&str>) -> String {
    if let Some(db_uri) = db_uri {
        db_uri.to_string()
    } else if let Ok(db_uri) = std::env::var("QUOTE_DB_URI") {
        db_uri
    } else {
        "sqlite://db/quotes.db".to_string()
    }
}

fn extract_db_dir(db_uri: &str) -> Result<&str, QuoteError> {
    if db_uri.starts_with("sqlite://") && db_uri.ends_with(".db") {
        let start = db_uri.find(':').unwrap() + 3;
        let mut path = &db_uri[start..];
        if let Some(end) = path.rfind('/') {
            path = &path[..end];
        } else {
            path = "";
        }
        Ok(path)
    } else {
        Err(QuoteError::InvalidDbUri(db_uri.to_string()))
    }
}


async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let db_uri = get_db_uri(args.db_uri.as_deref());
    if !sqlite::Sqlite::database_exists(&db_uri).await? {
        let db_dir = extract_db_dir(&db_uri)?;
        std::fs::create_dir_all(db_dir)?;
        sqlite::Sqlite::create_database(&db_uri).await?
    }

    let db = SqlitePool::connect(&db_uri).await?;

    sqlx::migrate!().run(&db).await?;
    if let Some(path) = args.init_from {
        let quotes = read_quotes(path)?;
        let mut tx = db.begin().await?;
        for q in &quotes {
            sqlx::query!(
                "INSERT INTO quotes VALUES ($1, $2);",
                q.words,
                q.author,
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
    }
    let state = Arc::new(RwLock::new(AppState { db }));

    /*
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "quote=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));
    */

    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();
    let app = axum::Router::new()
        .route("/", routing::get(get_quote))
        .route_service(
            "/quote.css",
            services::ServeFile::new_with_mime("assets/static/quote.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        //.layer(trace_layer)
        .with_state(state);

    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("quoteserver: error: {}", err);
        std::process::exit(1);
    }
}
