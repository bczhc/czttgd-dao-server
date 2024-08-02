use axum::response::{IntoResponse, Response};
use axum::{debug_handler, Extension};
use futures::TryStreamExt;
use log::info;

use crate::handlers::{handle_errors, User};
use crate::{api_ok, include_sql, ApiContext};

#[debug_handler]
pub async fn all_users(extension: Extension<ApiContext>) -> Response {
    info!("Route: /users");
    let result: anyhow::Result<()> = try {
        let db = &extension.db;
        let rows = sqlx::query_as::<_, User>(include_sql!("users")).fetch(db);
        let rows = rows.try_collect::<Vec<_>>().await?;
        return api_ok!(rows);
    };
    handle_errors!(result)
}
