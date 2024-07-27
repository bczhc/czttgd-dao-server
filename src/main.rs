#![feature(let_chains)]

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{Extension, Router};
use axum::routing::get;
use clap::Parser;
use log::{debug, info};
use once_cell::sync::Lazy;
use sqlx::{Executor, MySqlPool, Pool, Row, Statement};
use sqlx::mysql::MySqlTypeInfo;

use czttgd_dao::{ApiContext, ApiContextInner, Args, ARGS, CONFIG, DATABASE_NAME, handlers, mutex_lock, set_up_logging};

use czttgd_dao::config::{get_config, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = get_config(&args.config)?;
    if let Some(l) = &config.logging && let Some(f) = &l.file {
        set_up_logging(f)?;
    }
    debug!("Args: {:?}", args);
    debug!("Configs: {:?}", config);
    *mutex_lock!(ARGS) = args.clone();
    *mutex_lock!(CONFIG) = config.clone();

    info!("Connecting to the database...");
    let pool = MySqlPool::connect(
        format!(
            "mysql://{}:{}@{}:{}/{}",
            config.mysql.username,
            config.mysql.password,
            config.mysql.ip,
            config.mysql.port,
            DATABASE_NAME
        )
        .as_str(),
    )
    .await?;
    info!("Testing the connection...");
    let row: (i64,) = sqlx::query_as("SELECT ?")
        .bind(42_i64)
        .fetch_one(&pool)
        .await?;
    assert_eq!(row.0, 42);
    info!("Done.");

    start_axum(Arc::new(ApiContextInner { db: pool })).await?;

    Ok(())
}

fn router() -> Router {
    Router::new()
        .route("/echo", get(handlers::demo::echo))
        .nest("/", handlers::router())
}

async fn start_axum(api_context: ApiContext) -> anyhow::Result<()> {
    let listen_port = mutex_lock!(CONFIG).listen_port;

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), listen_port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let router = router().layer(Extension(api_context));
    info!("Server started on {}", addr);
    axum::serve(listener, router).await?;
    Ok(())
}
