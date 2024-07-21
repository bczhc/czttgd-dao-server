use axum::{Extension, Form};
use axum::response::IntoResponse;
use log::debug;
use sqlx::Executor;

use crate::{api_error, api_ok, ApiContext};
use crate::handlers::Inspection;

#[axum::debug_handler]
pub async fn post_new(
    Extension(api_context): Extension<ApiContext>,
    Form(record): Form<Inspection>,
) -> impl IntoResponse {
    debug!("post_new record: {:?}", record);
    let db = &api_context.db;

    let break_flag = record.break_type == 0;

    let result: anyhow::Result<()> = try {
        let query = sqlx::query(
            r"INSERT INTO tt_inspect
(creator, devicecode, creationtime, spec,
 wirenum, breakspec, twbatchcode, trbatchcode,
 dlwarehouse, breakflag,
 breakpointa, breakpointb, memo, devicecategory, billflag)
    VALUE (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0);",
        )
        .bind(record.creator)
        .bind(record.machine_number)
        .bind(record.creation_time)
        .bind(record.product_specs)
        .bind(record.wire_number)
        .bind(record.break_specs)
        .bind(record.copper_wire_no)
        .bind(record.copper_stick_no)
        .bind(record.repo_no)
        .bind(break_flag)
        .bind(record.break_position_a)
        .bind(record.break_position_b)
        .bind(record.comments)
        .bind(record.machine_category);
        db.execute(query).await?;
        return api_ok!(());
    };
    debug!("Result: {:?}", result);
    api_error!()
}
