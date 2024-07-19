use axum::{debug_handler, Extension, extract};
use axum::response::{IntoResponse, Response};
use clap::builder::TypedValueParser;
use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;

use crate::{api_error, api_ok, ApiContext, ResponseJson};

use std::sync::Arc;

#[derive(Deserialize)]
pub struct Path {
    stage: i32,
}
#[debug_handler]
pub async fn machines(
    extract::Path(stage): extract::Path<Path>,
    extension: Extension<ApiContext>,
) -> Response {
    let stage = stage.stage;

    let _: anyhow::Result<()> = try {
        #[derive(sqlx::FromRow)]
        struct Row {
            machine_no: i32,
        }

        let db = &extension.db;
        let mut machines =
            sqlx::query_as::<_, Row>("SELECT machine_no FROM tt_machine WHERE stage = ?")
                .bind(stage)
                .fetch(db);
        let machines = machines
            .map(|x| x.map(|x| x.machine_no))
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        return api_ok!(machines);
    };

    api_error!()
}
