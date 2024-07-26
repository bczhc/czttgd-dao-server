#![feature(try_blocks)]
#![feature(decl_macro)]
#![feature(yeet_expr)]

use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use axum::Json;
use axum::response::IntoResponse;
use serde::Serialize;
use sqlx::{FromRow, MySql, Pool, Row};
use sqlx::mysql::MySqlRow;

pub mod handlers;

pub const DATABASE_NAME: &str = "breakInfo";

#[derive(clap::Parser, Debug, Default, Clone)]
pub struct Args {
    pub mysql_server: String,
    /// Path to MySql credentials file
    ///
    /// Format:
    /// 1| <username>
    /// 2| <password>
    pub mysql_credentials_file: PathBuf,
    /// Port the HTTP server will listen on
    #[arg(short = 'p', long, default_value = "8010")]
    pub listen_port: u16,
    #[arg(short = 'P', long, default_value = "3306")]
    pub mysql_port: u16,
}

pub fn set_up_logging() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(io::stdout())
        .chain(fern::log_file("czttgd-dao.log")?)
        .apply()?;
    Ok(())
}

pub fn read_credentials<P: AsRef<Path>>(path: P) -> io::Result<(String, String)> {
    let reader = BufReader::new(File::open(path.as_ref())?);
    let mut lines = reader.lines();
    let credentials: Option<(io::Result<String>, io::Result<String>)> = try {
        let username = lines.next()?;
        let password = lines.next()?;
        (username, password)
    };
    let results =
        credentials.ok_or_else(|| io::Error::other(anyhow!("Malformed credential file")))?;
    Ok((results.0?, results.1?))
}

#[derive(Serialize)]
pub struct ResponseJson<D: Serialize> {
    data: Option<D>,
    code: u32,
    message: Option<String>,
}

impl<D: Serialize> ResponseJson<D> {
    pub fn ok(data: D) -> Self {
        Self {
            data: Some(data),
            code: 0,
            message: None,
        }
    }

    pub fn error() -> Self {
        Self {
            data: None,
            code: 1,
            message: None,
        }
    }

    pub fn error_msg<S: Into<String>>(message: S) -> Self {
        Self {
            data: None,
            code: 1,
            message: Some(message.into()),
        }
    }
}

impl<D: Serialize> IntoResponse for ResponseJson<D> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

pub struct ApiContextInner {
    pub db: Pool<MySql>,
}

pub type ApiContext = Arc<ApiContextInner>;

pub macro mutex_lock($e:expr) {
    $e.lock().unwrap()
}

pub macro api_ok($d:expr) {
    crate::ResponseJson::ok($d).into_response()
}

fn check_from_row<'a, T, F>(r: &'a MySqlRow, ref_name: &str, get_fn: F) -> sqlx::Result<Option<T>>
where
    T: FromRow<'a, MySqlRow>,
    F: FnOnce(&'a MySqlRow) -> sqlx::Result<T>,
{
    try {
        if r.try_get::<Option<i32>, _>(ref_name)?.is_some() {
            Some(get_fn(r)?)
        } else {
            None
        }
    }
}

pub macro include_sql($name:literal) {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/sqls/", $name, ".sql"))
}

// their SQL uses `INT` but not `INT UNSIGNED`
pub type RefId = i32;
