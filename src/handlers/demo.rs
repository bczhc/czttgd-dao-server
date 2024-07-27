use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Json;
use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Input {
    text: String,
}

pub async fn echo(query: Query<Input>) -> impl IntoResponse {
    debug!("Route: echo");
    Json(query.0)
}
