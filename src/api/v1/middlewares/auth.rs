#![cfg(feature = "database")]

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json, body::Body,
};

use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Serialize;

use crate::api::api_v1::{ApiV1State, models::user::{JwtUser, User}, services::users::UserPDO};



#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

pub async fn auth(
    cookie_jar: CookieJar,
    State(data): State<Arc<ApiV1State<'_>>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let token = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .map(|auth_value| {
                    auth_value.trim_start_matches("Bearer").trim().to_string()
                })
        });

    let token = token.ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "You are not logged in, please provide token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    let claims = decode::<JwtUser>(
        &token,
        &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?
    .claims;

    let _ = uuid::Uuid::parse_str(&claims.id).map_err(|_| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    let password_rev = uuid::Uuid::parse_str(&claims.password_rev).map_err(|_| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    let user = UserPDO::fetch_user_by_id(&data, &claims.id)
    .await
    .map_err(|err| {
        let json_error = ErrorResponse {
            status: "fail",
            message: format!("Internal server error: {}", err),
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
    })?;


    let user = user.ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid Token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    // check password rev
    if user.password_rev != password_rev {
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid token".to_string(),
        };
        return Err((StatusCode::UNAUTHORIZED, Json(json_error)));
    }

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

// middleware is_admin 
pub async fn is_admin(
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let user = req.extensions().get::<User>().ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "You are not logged in, please provide token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    if !user.is_admin() {
        let json_error = ErrorResponse {
            status: "fail",
            message: "You are not authorized to perform this action".to_string(),
        };
        return Err((StatusCode::FORBIDDEN, Json(json_error)));
    }

    Ok(next.run(req).await)
}