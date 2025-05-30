mod error;
mod quote;
mod templates;
mod web;
mod api;

use error::*;
use quote::*;
use templates::*;

extern crate log;
extern crate mime;

use axum::{
    self,
    extract::{Path, Query, State, Json},
    http,
    response::{self, IntoResponse},
    routing,
};
use clap::Parser;
extern crate fastrand;
use serde::{Serialize, Deserialize};
use sqlx::{Row, SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use std::borrow::Cow;
use std::sync::Arc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
    #[arg(short, long, default_value = "3000")]
    port: u16,
}

struct AppState {
    db: SqlitePool,
    current_quote: Quote,
}

fn get_db_uri(db_uri: Option<&str>) -> Cow<str> {
    if let Some(db_uri) = db_uri {
        db_uri.into()
    } else if let Ok(db_uri) = std::env::var("DATABASE_URL") {
        db_uri.into()
    } else {
        "sqlite://db/quotes.db".into()
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
        'next_quote: for jj in quotes {
            let mut jtx = db.begin().await?;
            let (j, ts) = jj.to_quote();
            let quote_insert = sqlx::query!(
                "INSERT INTO quotes (id, words, author, quote_source) VALUES ($1, $2, $3, $4);",
                j.id,
                j.words,
                j.author,
                j.quote_source,
            )
            .execute(&mut *jtx)
            .await;
            if let Err(e) = quote_insert {
                eprintln!("error: quote insert: {}: {}", j.id, e);
                jtx.rollback().await?;
                continue;
            };
            for t in ts {
                let tag_insert =
                    sqlx::query!("INSERT INTO tags (quote_id, tag) VALUES ($1, $2);", j.id, t,)
                        .execute(&mut *jtx)
                        .await;
                if let Err(e) = tag_insert {
                    eprintln!("error: tag insert: {} {}: {}", j.id, t, e);
                    jtx.rollback().await?;
                    continue 'next_quote;
                };
            }
            jtx.commit().await?;
        }
        return Ok(());
    }
    let current_quote = Quote {
        id: "test".to_string(),
        words: "Test Quote".to_string(),
        author: "Test Author".to_string(),
        quote_source: "Unknown".to_string(),
    };
    let app_state = AppState { db, current_quote };
    let state = Arc::new(RwLock::new(app_state));

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

    let cors = tower_http::cors::CorsLayer::new()
        .allow_methods([http::Method::GET])
        .allow_origin(tower_http::cors::Any);

    async fn handler_404() -> axum::response::Response {
        (http::StatusCode::NOT_FOUND, "404 Not Found").into_response()
    }

    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();

    let (api_router, api) = OpenApiRouter::with_openapi(api::ApiDoc::openapi())
        .nest("/api/v1", api::router())
        .split_for_parts();

    let swagger_ui = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", api.clone());
    let redoc_ui = Redoc::with_url("/redoc", api);
    let rapidoc_ui = RapiDoc::new("/api-docs/openapi.json").path("/rapidoc");



    let app = axum::Router::new()
        .route("/", routing::get(web::get_quote))
        .route_service(
            "/quote.css",
            services::ServeFile::new_with_mime("assets/static/quote.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        .merge(swagger_ui)
        .merge(redoc_ui)
        .merge(rapidoc_ui)
        .merge(api_router)
        .fallback(handler_404)
        .layer(cors)
        .layer(trace_layer)
        .with_state(state);

    let listener = net::TcpListener::bind(&format!("127.0.0.1:{}", args.port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("quote: error: {}", err);
        std::process::exit(1);
    }
}