use anyhow::anyhow;
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use axum::{Extension, Form};
use futures::TryStreamExt;
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::database::HasArguments;
use sqlx::mysql::MySqlArguments;
use sqlx::types::BigDecimal;
use sqlx::{Database, Executor, FromRow, MySql, Row};
use yeet_ops::yeet;

use crate::handlers::{
    handle_errors, BreakCause, Breakpoint, InspectionDetails, InspectionForm, InspectionSummary,
    User,
};
use crate::{api_ok, check_from_row, include_sql, ApiContext};

#[axum::debug_handler]
pub async fn post_new(
    Extension(api_context): Extension<ApiContext>,
    Form(form): Form<InspectionForm>,
) -> impl IntoResponse {
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let query = sqlx::query(include_sql!("inspection-post"));
        let query = bind_form(query, form);
        let r = db.execute(query).await?;
        let last_id = r.last_insert_id();

        return api_ok!(last_id);
    };
    handle_errors!(result)
}

fn bind_form(
    query: sqlx::query::Query<MySql, MySqlArguments>,
    form: InspectionForm,
) -> sqlx::query::Query<MySql, MySqlArguments> {
    // They use `char(1)`. According the conversion,
    // we must insert a `&str`.
    let break_flag = match form.break_flag {
        true => "1",
        false => "0",
    };

    return query
        .bind(form.creator)
        .bind(form.device_code)
        .bind(form.creation_time)
        .bind(form.product_spec)
        .bind(form.wire_speed)
        .bind(form.wire_number)
        .bind(form.break_spec)
        .bind(form.wire_batch_code)
        .bind(form.stick_batch_code)
        .bind(form.warehouse)
        .bind(break_flag)
        .bind(form.breakpoint_a)
        .bind(form.breakpoint_b)
        .bind(form.comments)
        .bind(form.device_category);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchQuery {
    filter: String,
    stage: u32,
}

/// Performs a search, and returns a set of [`InspectionSummary`].
#[axum::debug_handler]
pub async fn search(
    Extension(api_context): Extension<ApiContext>,
    Query(api_query): Query<SearchQuery>,
) -> impl IntoResponse {
    let db = &api_context.db;

    let r: anyhow::Result<()> = try {
        let mut query = sqlx::query(include_sql!("inspection-search"));
        query = query.bind(api_query.stage as i32);
        for _ in 0..7 {
            query = query.bind(&api_query.filter);
        }
        let mut stream = query.fetch(db);
        let mut collected = vec![];
        while let Some(row) = stream.try_next().await? {
            let summary = InspectionSummary::from_row(&row)?;
            let summary = InspectionSummary {
                break_cause_a: check_from_row(&row, "br_a", |r| {
                    BreakCause::from_row_prefixed(r, "br_a")
                })?,
                break_cause_b: check_from_row(&row, "br_a", |r| {
                    BreakCause::from_row_prefixed(r, "br_b")
                })?,
                ..summary
            };
            collected.push(summary);
        }
        return api_ok!(collected);
    };
    handle_errors!(r)
}

#[axum::debug_handler]
pub async fn query_details(
    Extension(api_context): Extension<ApiContext>,
    path: Path<(u32,)>,
) -> impl IntoResponse {
    let id = path.0 .0;
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let r = sqlx::query(include_sql!("inspection-details"))
            .bind(id)
            .fetch_one(db)
            .await?;

        let details = InspectionDetails::from_row(&r)?;
        let details = InspectionDetails {
            breakpoint_a: check_from_row(&r, "breakpoint_a", |r| {
                Breakpoint::from_row_prefixed(r, "bp_a")
            })?,
            break_cause_a: check_from_row(&r, "break_cause_a", |r| {
                BreakCause::from_row_prefixed(r, "br_a")
            })?,
            break_cause_b: check_from_row(&r, "break_cause_b", |r| {
                BreakCause::from_row_prefixed(r, "br_b")
            })?,
            inspector: check_from_row(&r, "inspector", |r| {
                User::from_row_prefixed(r, "inspector")
            })?,
            ..details
        };
        return api_ok!(details);
    };
    handle_errors!(result)
}

#[axum::debug_handler]
pub async fn update(
    Extension(api_context): Extension<ApiContext>,
    Path(path): Path<(u32,)>,
    Form(form): Form<InspectionForm>,
) -> impl IntoResponse {
    let id = path.0;
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let query = sqlx::query(include_sql!("inspection-update"));
        let query = bind_form(query, form);
        // WHERE id = ?
        let query = query.bind(id as i32);
        query.execute(db).await?;
        return api_ok!(());
    };
    handle_errors!(result)
}
