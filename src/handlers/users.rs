use axum::{debug_handler, Extension};
use axum::response::{IntoResponse, Response};
use futures::TryStreamExt;
use serde::Serialize;

use crate::{api_error, api_ok, ApiContext};

#[debug_handler]
pub async fn all_users(extension: Extension<ApiContext>) -> Response {
    let _: anyhow::Result<()> = try {
        #[derive(sqlx::FromRow, Serialize)]
        struct Row {
            name: String,
            username: String,
        }

        let db = &extension.db;
        let rows = sqlx::query_as::<_, Row>("SELECT name, username FROM tt_user").fetch(db);
        let rows = rows.try_collect::<Vec<_>>().await.unwrap();
        return api_ok!(rows);
    };
    api_error!()
}
