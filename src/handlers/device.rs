use axum::response::{IntoResponse, Response};
use axum::{debug_handler, extract, Extension};
use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;
use sqlx::Row;

use crate::handlers::handle_errors;
use crate::{api_ok, include_sql, ApiContext};

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
