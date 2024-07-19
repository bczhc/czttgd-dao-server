use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{debug_handler, Router};
use axum::routing::{get, post};
use clap::builder::TypedValueParser;
use futures::TryStreamExt;

use crate::{mutex_lock, ResponseJson};

pub mod demo;
pub mod machines;

pub fn router() -> Router {
    Router::new().route("/stage/:stage/machines", get(machines::machines))
}


