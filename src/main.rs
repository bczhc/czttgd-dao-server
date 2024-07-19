use std::net::SocketAddr;
use std::sync::Mutex;
use axum::Router;
use axum::routing::get;

use clap::Parser;
use log::{debug, info};
use once_cell::sync::Lazy;
use sqlx::{Executor, MySqlPool, Row, Statement};
use sqlx::mysql::MySqlTypeInfo;

use czttgd_dao::{Args, DATABASE_NAME, mutex_lock, read_credentials, handlers, set_up_logging};

static ARGS: Lazy<Mutex<Args>> = Lazy::new(|| Mutex::new(Default::default()));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_up_logging()?;

    let args = Args::parse();
    *mutex_lock!(ARGS) = args.clone();

    debug!("Args: {:?}", args);

    let credentials = read_credentials(&args.mysql_credentials_file)?;

    info!("Connecting to the database...");
    let pool = MySqlPool::connect(format!("mysql://{}:{}@{}:{}/{}", credentials.0, credentials.1, args.mysql_server, args.mysql_port, DATABASE_NAME).as_str()).await?;
    info!("Testing the connection...");
    let row: (i64,) = sqlx::query_as("SELECT ?")
        .bind(42_i64)
        .fetch_one(&pool).await?;
    assert_eq!(row.0, 42);
    info!("Done.");
    
    start_axum().await?;

    Ok(())
}

async fn start_axum() -> anyhow::Result<()> {
    let listen_port = mutex_lock!(ARGS).listen_port;
    
    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), listen_port);

    let router = Router::new()
        .route("/echo", get(handlers::demo::echo));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    info!("Server started on {}", addr);
    Ok(())
}
