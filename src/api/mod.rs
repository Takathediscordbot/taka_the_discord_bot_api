pub mod v1;

use std::fmt::Display;

use axum::Router;
pub use v1 as api_v1;

#[derive(Debug)]
pub struct Error(pub String);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}


pub async fn api() -> Result<Router, Error> {
    let api = 
        Router::new()
            .nest("/v1", v1::api_v1().await?);

    Ok(api)
}