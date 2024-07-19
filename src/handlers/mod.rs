use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use clap::builder::TypedValueParser;
use futures::TryStreamExt;

pub mod demo;
mod machines;
mod users;

pub fn router() -> Router {
    Router::new()
        .route("/stage/:stage/machines", get(machines::machines))
        .route("/users", get(users::all_users))
}
