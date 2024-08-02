use std::time::{SystemTime, UNIX_EPOCH};

use axum::{Extension, Form};
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use futures::TryStreamExt;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, MySql, Row};
use sqlx::mysql::MySqlArguments;

use crate::{api_ok, ApiContext, check_from_row, include_sql, MySqlPool, timestamp_secs};
use crate::handlers::{BreakCause, Breakpoint, counter, handle_errors, InspectionDetails, InspectionForm, InspectionSummary, User};

/// timestamp+<counter>
///
/// `counter` has three-width padding, like 001, 002...
async fn generate_inspection_id(db: &MySqlPool) -> anyhow::Result<i64> {
    let id_num = counter::increase(db).await?;

    let timestamp = timestamp_secs();
    let id = format!("{timestamp}{:03}", id_num).parse::<i64>()?;
    Ok(id)
}

#[axum::debug_handler]
pub async fn post_new(
    Extension(api_context): Extension<ApiContext>,
    Form(form): Form<InspectionForm>,
) -> impl IntoResponse {
    info!("Route: /inspection");
    debug!("Form: {:?}", form);
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let id = generate_inspection_id(db).await?;

        let query = sqlx::query(include_sql!("inspection-post"));
        let query = bind_form(query, form);
        // bind: id
        let query = query.bind(id);
        db.execute(query).await?;

        return api_ok!(id);
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
        .bind(form.wire_number)
        .bind(form.break_spec)
        .bind(form.wire_batch_code)
        .bind(form.stick_batch_code)
        .bind(form.warehouse)
        .bind(break_flag)
        .bind(form.breakpoint_a)
        .bind(form.breakpoint_b)
        .bind(form.comments)
        .bind(form.device_category)
        .bind(form.break_cause_a);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchQuery {
    filter: String,
    stage: u32,
    limit: Option<u64>,
    offset: Option<u64>,
}

/// Performs a search, and returns a set of [`InspectionSummary`].
#[axum::debug_handler]
pub async fn search(
    Extension(api_context): Extension<ApiContext>,
    Query(api_query): Query<SearchQuery>,
) -> impl IntoResponse {
    info!("Route: /inspection/search");
    debug!("Query: {:?}", api_query);
    let db = &api_context.db;
    let offset = api_query.offset.unwrap_or_default();
    let limit = api_query.limit.unwrap_or(100);

    let r: anyhow::Result<()> = try {
        let mut query = sqlx::query(include_sql!("inspection-search"));
        query = query.bind(api_query.stage as i32);
        for _ in 0..10 {
            query = query.bind(&api_query.filter);
        }
        // limit, offset
        let query = query.bind(limit).bind(offset);
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
    path: Path<(i64,)>,
) -> impl IntoResponse {
    info!("Route: /inspection/:id/details");
    debug!("Path: {:?}", path);
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
            inspector: check_from_row(&r, "inspector_user_name", |r| {
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
    Path(path): Path<(i64,)>,
    Form(form): Form<InspectionForm>,
) -> impl IntoResponse {
    info!("Route: /inspection/:id");
    let id = path.0;
    let db = &api_context.db;

    let result: anyhow::Result<()> = try {
        let query = sqlx::query(include_sql!("inspection-update"));
        let query = bind_form(query, form);
        // WHERE id = ?
        let query = query.bind(id);
        query.execute(db).await?;
        return api_ok!(());
    };
    handle_errors!(result)
}

#[axum::debug_handler]
pub async fn count(
    Extension(api_context): Extension<ApiContext>,
) -> impl IntoResponse {
    info!("Route: /inspection/count");
    let db = &api_context.db;
    let r: anyhow::Result<()> = try {
        let r = sqlx::query(include_sql!("inspection-count")).fetch_one(db).await?;
        let count: i64 = r.try_get("c")?;
        
        return api_ok!(count);
    };
    handle_errors!(r)
}
