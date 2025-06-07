use leptos::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize)]
pub struct Quote {
    pub id: String,
    pub words: String,
    pub author: String,
    pub tags: HashSet<String>,
    pub source: String,
}


pub async fn fetch(endpoint: String) -> Result<Quote, Error> {
    use reqwasm::http::Request;

    let ep = format!(
        "http://localhost:3000/api/v1/{}",
        endpoint,
    );
    let result = Request::get(&ep)
        .send()
        .await?
        // convert it to JSON
        .json()
        .await?;
    Ok(result)
}