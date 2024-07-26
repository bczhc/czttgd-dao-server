use std::sync::Arc;

use axum::{debug_handler, Extension, extract};
use axum::response::{IntoResponse, Response};
use clap::builder::TypedValueParser;
use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;
use sqlx::Row;

use crate::{api_ok, ApiContext, include_sql, ResponseJson};
use crate::handlers::handle_errors;

#[derive(Deserialize)]
pub struct Path {
    stage: i32,
}
#[debug_handler]
pub async fn devices(
    extract::Path(stage): extract::Path<Path>,
    extension: Extension<ApiContext>,
) -> Response {
    let stage = stage.stage;

    let result: anyhow::Result<()> = try {
        let db = &extension.db;
        let devices = sqlx::query(include_sql!("devices")).bind(stage).fetch(db);
        let devices = devices
            .map(|x| x.map(|x| x.get::<i32, _>(0)))
            .try_collect::<Vec<_>>()
            .await?;
        return api_ok!(devices);
    };
    handle_errors!(result)
}
