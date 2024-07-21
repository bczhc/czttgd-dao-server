use axum::{debug_handler, Extension, extract};
use axum::response::{IntoResponse, Response};
use clap::builder::TypedValueParser;
use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;

use crate::{api_error, api_ok, ApiContext, ResponseJson};

use std::sync::Arc;
use sqlx::Row;

#[derive(Deserialize)]
pub struct Path {
    stage: i32,
}
#[debug_handler]
pub async fn machines(
    extract::Path(stage): extract::Path<Path>,
    extension: Extension<ApiContext>,
) -> Response {
    let stage = stage.stage;

    let _: anyhow::Result<()> = try {
        let db = &extension.db;
        let mut machines =
            sqlx::query("SELECT machinenumber FROM tt_machine WHERE stage = ?")
                .bind(stage)
                .fetch(db);
        let machines = machines.map(|x| x.map(|x| x.get::<u32, _>(0))).try_collect::<Vec<_>>().await?;
        return api_ok!(machines);
    };

    api_error!()
}
