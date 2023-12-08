pub mod v1;

use axum::Router;
pub use v1 as api_v1;

use crate::Error;

pub async fn api() -> Result<Router, Error> {
    let api = 
        Router::new()
            .nest("/v1", v1::api_v1().await?);

    Ok(api)
}