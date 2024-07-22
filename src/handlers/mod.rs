use std::sync::Mutex;
use std::{fmt, io};

use axum::response::IntoResponse;
use axum::Router;
use clap::builder::TypedValueParser;
use futures::TryStreamExt;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::types::BigDecimal;

use crate::mutex_lock;

mod breakpoint;
pub mod demo;
mod inspection;
mod machines;
mod users;

static COLLECTED_ROUTES: Lazy<Mutex<Vec<&'static str>>> =
    Lazy::new(|| Mutex::new(Default::default()));

macro add_route($router:expr, $t:tt $path:literal, $f:expr) {
    paste::paste! {
        $router = $router.route($path, ::axum::routing::[<$t:lower>]($f));
        mutex_lock!(COLLECTED_ROUTES).push(concat!(stringify!([<$t:upper>]), " ", $path));
    }
}

pub fn router() -> Router {
    let mut router = Router::new();
    add_route!(router, GET "/routes", list_routes);
    add_route!(router, GET "/stage/:stage/machines", machines::machines);
    add_route!(router, GET "/users", users::all_users);
    add_route!(router, GET "/break/reasons", breakpoint::all_break_reasons);
    add_route!(router, GET "/break/points", breakpoint::all_breakpoints);
    add_route!(router, POST "/inspection", inspection::post_new);
    add_route!(router, GET "/inspections", inspection::query_summary);
    add_route!(router, GET "/inspection/:id/details", inspection::query_details);
    // add_route!(router, PUT "/inspection/:id");
    router
}

pub async fn list_routes() -> impl IntoResponse {
    let mut content = String::new();
    use fmt::Write;
    for &line in &*mutex_lock!(COLLECTED_ROUTES) {
        writeln!(&mut content, "{}", line).unwrap();
    }
    content
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InspectionForm {
    pub creator: String,
    pub machine_number: u32,
    pub machine_category: String,
    pub creation_time: String,
    pub product_specs: String,
    pub wire_number: Option<u32>,
    pub break_specs: String,
    pub copper_wire_no: Option<u32>,
    pub copper_stick_no: Option<u32>,
    pub repo_no: Option<u32>,
    // 0: 拉丝池内断线
    // 1: 非拉丝池内断线
    pub break_type: u32,
    // 拉丝池
    pub break_position_b: Option<f32>,
    // 非拉丝池
    pub break_position_a: Option<String>,
    // 初检
    pub break_reason_a: String,
    pub comments: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InspectionDetails {
    device_code: u32,
    creator: String,
    creation_time: String,
    inspection_flag: u32,
    product_spec: Option<String>,
    wire_num: Option<u32>,
    break_spec: String,
    wire_batch_code: Option<String>,
    stick_batch_code: Option<String>,
    warehouse: Option<String>,
    product_time: Option<String>,
    break_flag: bool,
    breakpoint_b: Option<BigDecimal>,
    breakpoint_a: Option<String>,
    cause_type: Option<String>,
    break_cause_a: Option<String>,
    comments: Option<String>,
    inspector: Option<String>,
    inspection_time: Option<String>,
    break_cause_b: Option<String>,
}
