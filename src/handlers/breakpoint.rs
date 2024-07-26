use axum::{Extension, Form, Router};
use axum::response::IntoResponse;
use axum::routing::get;
use clap::builder::TypedValueParser;
use futures::{FutureExt, TryStreamExt};
use sqlx::{MySql, Row};
use sqlx::mysql::MySqlRow;

use crate::{api_ok, ApiContext, include_sql};
use crate::handlers::{BreakCause, Breakpoint, handle_errors, InspectionForm};

pub async fn all_break_reasons(Extension(api_context): Extension<ApiContext>) -> impl IntoResponse {
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let collected: Vec<BreakCause> = sqlx::query_as(include_sql!("break-reasons"))
            .fetch_all(db)
            .await?;
        return api_ok!(collected);
    };
    handle_errors!(result)
}

pub async fn all_breakpoints(Extension(api_context): Extension<ApiContext>) -> impl IntoResponse {
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let collected: Vec<Breakpoint> = sqlx::query_as(include_sql!("breakpoints"))
            .fetch_all(db)
            .await?;
        return api_ok!(collected);
    };
    handle_errors!(result)
}
