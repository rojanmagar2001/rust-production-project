use axum::{
    extract::{Path, Query},
    http::{Method, Uri},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use ctx::Ctx;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use crate::{log::log_request, model::ModelController};

pub use self::error::{Error, Result};

mod ctx;
mod error;
mod log;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time() // For early local development
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    // Initialize ModelController.
    let mc = ModelController::new().await?;

    let routes_api = web::routes_ticket::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        .nest("/api", routes_api)
        .layer(middleware::map_response(
            web::mw_res_map::main_response_mapper,
        ))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new());

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    info!("->> LISTENING on {:?}\n", listener.local_addr());
    axum::serve(listener, routes_all.into_make_service())
        .await
        .unwrap();

    Ok(())
}

// region: --- Routes Hello
fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/:name", get(handler_hello2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// e.g., `/hello?name=world`
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    debug!(" {:<12} handler_hello: {:?}", "HANDLER", params);

    let name = params.name.as_deref().unwrap_or("World!");
    Html(format!("Hello <strong>{}</strong>!", name))
}

// e.g., `/hello2/Rojan`
async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    debug!(" {:<12} handler_hello2: {:?}", "HANDLER", name);

    Html(format!("Hello <strong>{}</strong>!", name))
}
