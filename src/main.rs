use std::fmt::Display;
mod api;
use axum::{Router, response::IntoResponse};
use flexi_logger::{TS_DASHES_BLANK_COLONS_DOT_BLANK, DeferredNow, Logger, FileSpec, WriteMode};
use itertools::Itertools;
use log::Record;
use tower_http::cors::CorsLayer;

#[derive(Debug)]
pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

async fn run_server() -> Result<(), Error> {
    let ip_bind = std::env::var("BIND_URL").unwrap_or("0.0.0.0:8080".to_string());

    let cors = CorsLayer::permissive();
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .nest("/api", api::api().await?)
        .route("/health", axum::routing::get(health_status))
        .route("/logs",axum::routing::get(logs))       
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(&ip_bind).await.map_err(|e| {
        Error(format!("Couldn't bind to address {ip_bind}: {e}"))
    })?;
    // run our app with hyper
    let _ = axum::serve(listener, app)
            .await;

    Ok(())
}



pub fn my_own_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "{} [Thread {}] Severity {}, Message: {}",
        now.format(TS_DASHES_BLANK_COLONS_DOT_BLANK),
        std::thread::current().name().unwrap_or("<unnamed>"),
        record.level(),
        &record.args()
    )
}



async fn run() -> ! {
    dotenvy::dotenv().expect("Couldn't find env vars");
    let _logger = Logger::try_with_str("warn, taka_the_discord_bot_api=info").expect("Couldn't initialize logger")
    .log_to_file(FileSpec::default().directory("./logs"))
    .write_mode(WriteMode::BufferAndFlush)
    .format(my_own_format)
    .start().expect("Couldn't start logger");

    loop {
        match run_server().await {
            Ok(()) => continue,
            Err(Error(message)) => {
                log::error!("{message}");
                panic!("Couldn't run server! {message}")
            }
        }
    }
}

fn start() -> ! {
    tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
        run().await
    });

    panic!("Run loop has ended")
}
fn main() -> ! {
    start();  
}

async fn logs() -> impl IntoResponse {
    let last_modified_file = std::fs::read_dir("./logs")
    .expect("Couldn't access local directory")
    .flatten() // Remove failed
    .filter(|f| f.metadata().unwrap().is_file()) // Filter out directories (only consider files)
    .max_by_key(|x| x.metadata().unwrap().modified().unwrap()).unwrap(); // Get the most recently modified file

    let value = std::fs::read_to_string(format!("{}", last_modified_file.path().to_str().unwrap())).unwrap();
    let return_val: String = value.lines().map(|c| c.to_string()).join("\n");
    return_val
}

async fn health_status() -> impl IntoResponse {
    "OK"
}