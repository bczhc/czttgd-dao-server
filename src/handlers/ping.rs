use axum::extract::Query;
use axum::response::IntoResponse;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::api_ok;

#[derive(Deserialize)]
pub struct Input {
    text: String,
}

#[derive(Serialize)]
pub struct Pong {
    text: String,
}

pub async fn ping(query: Query<Input>) -> impl IntoResponse {
    debug!("Route: ping");
    api_ok!(Pong {
        text: query.text.clone()
    })
}
