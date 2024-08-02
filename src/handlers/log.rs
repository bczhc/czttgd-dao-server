use std::fs::{create_dir, File};
use std::io;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::anyhow;
use axum::body::Bytes;
use axum::extract::Multipart;
use axum::response::IntoResponse;
use log::info;

use crate::{api_ok, timestamp_secs};
use crate::handlers::{api_error, handle_errors};

pub async fn upload_log(mut multipart: Multipart) -> impl IntoResponse {
    info!("Route: /log");

    let result: anyhow::Result<()> = try {
        let on_error = || anyhow!("Unwrap None on multipart");
        let first = multipart.next_field().await?.ok_or_else(on_error)?;
        let name = first.name().ok_or_else(on_error)?;
        let bytes = first.bytes().await?;
        save_log(bytes)?;
        return api_ok!(());
    };
    handle_errors!(result)
}

fn save_log(log: Bytes) -> io::Result<()> {
    let log_dir = Path::new("./uploaded-log");
    if !log_dir.exists() {
        create_dir(log_dir)?;
    }
    let file_path = log_dir.join(format!("{}", timestamp_secs()));
    info!("Receive and write log: {}", file_path.display());
    let mut writer = BufWriter::new(File::create(file_path)?);
    writer.write_all(&log)?;
    Ok(())
}
