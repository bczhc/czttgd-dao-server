use axum::{debug_handler, Extension};
use axum::response::{IntoResponse, Response};
use futures::TryStreamExt;
use serde::Serialize;

use crate::{api_ok, ApiContext, include_sql};
use crate::handlers::{handle_errors, User};

#[debug_handler]
pub async fn all_users(extension: Extension<ApiContext>) -> Response {
    let result: anyhow::Result<()> = try {
        let db = &extension.db;
        let rows = sqlx::query_as::<_, User>(include_sql!("users")).fetch(db);
        let rows = rows.try_collect::<Vec<_>>().await.unwrap();
        return api_ok!(rows);
    };
    handle_errors!(result)
}
