use std::fmt;
use std::sync::Mutex;

use axum::response::IntoResponse;
use axum::Router;
use log::{debug, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlRow;
use sqlx::types::BigDecimal;
use sqlx::Row;

use crate::{mutex_lock, RefId};

mod breakpoint;
pub mod demo;
mod device;
pub mod inspection;
mod users;
mod counter;

#[path = "log.rs"]
mod log_router;
mod ping;

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
    add_route!(router, GET "/ping", ping::ping);
    add_route!(router, GET "/stage/:stage/devices", device::devices);
    add_route!(router, GET "/users", users::all_users);
    add_route!(router, GET "/break/causes", breakpoint::all_break_reasons);
    add_route!(router, GET "/break/points", breakpoint::all_breakpoints);
    add_route!(router, POST "/inspection", inspection::post_new);
    add_route!(router, GET "/inspection/search", inspection::search);
    add_route!(router, GET "/inspection/:id/details", inspection::query_details);
    add_route!(router, PUT "/inspection/:id", inspection::update);
    add_route!(router, GET "/inspection/count", inspection::count);
    add_route!(router, POST "/log", log_router::upload_log);
    router
}

pub async fn list_routes() -> impl IntoResponse {
    info!("Route: /routes");
    let mut content = String::new();
    use fmt::Write;
    for &line in &*mutex_lock!(COLLECTED_ROUTES) {
        writeln!(&mut content, "{}", line).unwrap();
    }
    content
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InspectionForm {
    pub creator: RefId,
    pub device_code: i32,
    pub device_category: String,
    pub creation_time: String,
    pub product_spec: Option<String>,
    pub wire_speed: Option<i32>,
    pub wire_number: Option<i32>,
    pub break_spec: String,
    pub wire_batch_code: Option<String>,
    pub stick_batch_code: Option<String>,
    pub warehouse: Option<String>,
    /// 是否拉丝池内断线
    pub break_flag: bool,
    /// 拉丝池 BigDecimal
    pub breakpoint_b: Option<String>,
    /// 非拉丝池 ref
    pub breakpoint_a: Option<RefId>,
    /// 初检原因
    pub break_cause_a: RefId,
    pub comments: Option<String>,
}

#[derive(Serialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InspectionDetails {
    id: i64,
    device_code: i32,
    device_category: String,
    /// ref
    #[sqlx(flatten)]
    creator: User,
    creation_time: String,
    inspection_flag: i32,
    product_spec: Option<String>,
    wire_speed: Option<i32>,
    wire_num: Option<i32>,
    break_spec: String,
    wire_batch_code: Option<String>,
    stick_batch_code: Option<String>,
    warehouse: Option<String>,
    product_time: Option<String>,
    break_flag: bool,
    breakpoint_b: Option<BigDecimal>,
    /// ref
    #[sqlx(skip)]
    breakpoint_a: Option<Breakpoint>,
    /// ref
    #[sqlx(skip)]
    break_cause_a: Option<BreakCause>,
    /// ref
    #[sqlx(skip)]
    break_cause_b: Option<BreakCause>,
    comments: Option<String>,
    /// ref
    #[sqlx(skip)]
    inspector: Option<User>,
    inspection_time: Option<String>,
}

#[derive(Serialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InspectionSummary {
    id: i64,
    device_code: i32,
    #[sqlx(skip)]
    break_cause_a: Option<BreakCause>,
    #[sqlx(skip)]
    break_cause_b: Option<BreakCause>,
    break_flag: bool,
    break_spec: String,
    product_spec: Option<String>,
    #[sqlx(flatten)]
    creator: User,
    creation_time: String,
    /// 0: 已初检 1: 已终检
    inspection_flag: i32,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[sqlx(rename = "user_id")]
    id: i32,
    #[sqlx(rename = "user_name")]
    name: String,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    #[sqlx(rename = "bp_id")]
    id: i32,
    #[sqlx(rename = "bp_name")]
    breakpoint: Option<String>,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakCause {
    #[sqlx(rename = "cause_id")]
    id: i32,
    #[sqlx(rename = "cause_type")]
    r#type: Option<String>,
    #[sqlx(rename = "cause_name")]
    cause: Option<String>,
}

impl BreakCause {
    pub fn from_row_prefixed(row: &MySqlRow, prefix: &str) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get(format!("{prefix}_cause_id").as_str())?,
            r#type: row.try_get(format!("{prefix}_cause_type").as_str())?,
            cause: row.try_get(format!("{prefix}_cause_name").as_str())?,
        })
    }
}

impl User {
    pub fn from_row_prefixed(row: &MySqlRow, prefix: &str) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get(format!("{prefix}_user_id").as_str())?,
            name: row.try_get(format!("{prefix}_user_name").as_str())?,
        })
    }
}

impl Breakpoint {
    pub fn from_row_prefixed(row: &MySqlRow, prefix: &str) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get(format!("{prefix}_bp_id").as_str())?,
            breakpoint: row.try_get(format!("{prefix}_bp_name").as_str())?,
        })
    }
}

pub macro api_error {
() => {
        crate::ResponseJson::<()>::error().into_response()
    },
($message:expr) => {{
        log::debug!("Error message:\n{}", $message);
        crate::ResponseJson::<()>::error_msg($message).into_response()
    }}
}

pub macro handle_errors($r:expr) {{
    log::debug!("Result: {:?}", &$r);
    let err = $r.err().unwrap();
    api_error!(format!("{}", err))
}}
