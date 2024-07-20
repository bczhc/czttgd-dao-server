use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use clap::builder::TypedValueParser;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

pub mod demo;
mod machines;
mod users;
mod breakpoint;

pub fn router() -> Router {
    Router::new()
        .route("/stage/:stage/machines", get(machines::machines))
        .route("/users", get(users::all_users))
        .route("/break/reasons", get(breakpoint::all_break_reasons))
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    pub creator: String,
    machine_number: u32,
    creation_time: String,
    product_specs: String,
    wire_number: u32,
    break_specs: String,
    copper_wire_no: Option<u32>,
    copper_stick_no: Option<u32>,
    repo_no: Option<u32>,
    // 0: 拉丝池内断线
    // 1: 非拉丝池内断线
    break_type: u32,
    break_position: String,
    // 初检
    break_reason_a: String,
    comments: Option<String>,
}
