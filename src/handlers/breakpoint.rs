use axum::response::IntoResponse;
use axum::Extension;
use log::info;

use crate::handlers::{handle_errors, BreakCause, Breakpoint};
use crate::{api_ok, include_sql, ApiContext};

pub async fn all_break_reasons(Extension(api_context): Extension<ApiContext>) -> impl IntoResponse {
    info!("Route: /break/causes");
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
    info!("Route: /break/points");
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let collected: Vec<Breakpoint> = sqlx::query_as(include_sql!("breakpoints"))
            .fetch_all(db)
            .await?;
        return api_ok!(collected);
    };
    handle_errors!(result)
}
