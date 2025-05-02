mod quote;
mod templates;

use quote::*;
use templates::*;
//extern crate mime;

use axum::{self, response, routing};
use tokio::net;
use tower_http::services;

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

const DB_URI: &str = "sqlite://quotes.db";

//returns a quote as a html response string
async fn get_quote() -> response::Html<String> {
    let quote = IndexTemplate::quote(&THE_QUOTE);
    response::Html(quote.to_string())
}


async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    //connect to sqlite database
    if !Sqlite::database_exists(DB_URI).await? {
        Sqlite::create_database(DB_URI).await?
    }
    let db = SqlitePool::connect(DB_URI).await?;
    //sqlx::migrate!().run(&db).await?;

    //create axum router with endpoint / that gets a quote
    let app = axum::Router::new()
        .route("/", routing::get(get_quote))
        .route_service(
            "/quote.css",
            services::ServeFile::new_with_mime("assets/static/quote.css", &mime::TEXT_CSS),
        );
    //bind IP address
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