use std::{fmt, io};
use std::sync::Mutex;

use axum::response::IntoResponse;
use axum::Router;
use clap::builder::TypedValueParser;
use futures::TryStreamExt;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::mutex_lock;

mod breakpoint;
pub mod demo;
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
