use axum::{Extension, Router};
use axum::response::IntoResponse;
use axum::routing::get;
use clap::builder::TypedValueParser;
use futures::{FutureExt, TryStreamExt};
use sqlx::{MySql, Row};
use sqlx::mysql::MySqlRow;

use crate::{api_error, api_ok, ApiContext};

pub async fn all_break_reasons(Extension(api_context): Extension<ApiContext>) -> impl IntoResponse {
    let db = &api_context.db;

    let _: anyhow::Result<()> = try {
        let mut collected = vec![];
        let mut rows = sqlx::query("SELECT tt_breakreason.breakreason FROM tt_breakreason;")
            .fetch(db);
        while let Some(r) = rows.try_next().await? {
            let reason: String = r.try_get(0)?;
            collected.push(reason);
        }
        return api_ok!(collected);
    };
    api_error!()
}
