use crate::*;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "quote-server", description = "Quote API")
    )
)]
pub struct ApiDoc;

pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new()
        .routes(routes!(get_quote))
        .routes(routes!(get_tagged_quote))
        .routes(routes!(get_random_quote))
        .routes(routes!(register))
        .routes(routes!(add_quote))
}

async fn get_quote_by_id(db: &SqlitePool, quote_id: &str) -> Result<response::Response, http::StatusCode> {
    let quote_result = quote::get(db, quote_id).await;
    match quote_result {
        Ok((quote, tags)) => Ok(JsonQuote::new(quote, tags).into_response()),
        Err(e) => {
            log::warn!("quote fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    get,
    path = "/quote/{quote_id}",
    responses(
        (status = 200, description = "Get a quote by id", body = [JsonQuote]),
        (status = 404, description = "No matching quote"),
    )
)]
pub async fn get_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(quote_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    get_quote_by_id(db, &quote_id).await
}

#[utoipa::path(
    get,
    path = "/tagged-quote",
    responses(
        (status = 200, description = "Get a quote by tags", body = [JsonQuote]),
        (status = 404, description = "No matching quotes"),
    )
)]
pub async fn get_tagged_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(tags): Json<Vec<String>>,
) -> Result<response::Response, http::StatusCode> {
    log::info!("get tagged quote: {:?}", tags);
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let quote_result = quote::get_tagged(db, tags.iter().map(String::as_ref)).await;
    match quote_result {
        Ok(Some(quote_id)) => get_quote_by_id(db, &quote_id).await,
        Ok(None) => {
            log::warn!("quote tag fetch failed tagging");
            Err(http::StatusCode::NOT_FOUND)
        }
        Err(e) => {
            log::warn!("quote tag fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    get,
    path = "/random-quote",
    responses(
        (status = 200, description = "Get a random quote", body = [JsonQuote]),
        (status = 404, description = "No quote"),
    )
)]
pub async fn get_random_quote(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let quote_result= quote::get_random(db).await;
    match quote_result {
        Ok(quote_id) => get_quote_by_id(db, &quote_id).await,
        Err(e) => {
            log::warn!("get random quote failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

#[utoipa::path(
    post,
    path = "/register",
    request_body(
        content = inline(authjwt::Registration),
        description = "Get an API key",
    ),
    responses(
        (status = 200, description = "JSON Web Token", body = authjwt::AuthBody),
        (status = 401, description = "Registration failed", body = authjwt::AuthError),
    )
)]
pub async fn register(
    State(appstate): State<SharedAppState>,
    Json(registration): Json<authjwt::Registration>,
) -> axum::response::Response {
    let appstate = appstate.read().await;
    match authjwt::make_jwt_token(&appstate, &registration) {
        Err(e) => e.into_response(),
        Ok(token) => (StatusCode::OK, token).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/add-quote",
    request_body(
        content = inline(JsonQuote),
        description = "Quote to add"
    ),
    responses(
        (status = 201, description = "Added quote", body = ()),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Auth Error", body = authjwt::AuthError),
    )
)]
pub async fn add_quote(
    _claims: authjwt::Claims,
    State(appstate): State<SharedAppState>,
    Json(quote): Json<JsonQuote>,
) -> axum::response::Response {
    let appstate = appstate.read().await;
    match quote::add(&appstate.db, quote).await {
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        Ok(()) => StatusCode::CREATED.into_response(),
    }
}