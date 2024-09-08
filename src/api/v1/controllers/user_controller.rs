#![cfg(feature = "database")]

use std::sync::Arc;

use axum::{extract::{Path, State}, http::{header, StatusCode}, response::{IntoResponse, Response}, routing::get, Extension, Json, Router};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use uuid::Uuid;

use crate::api::api_v1::{models::user::{FilteredUser, User, RegisterUserSchema, LoginUserSchema, JwtUser, UpdatePasswordSchema, UpdateUser, UpdateUserData, ForceUpdateUser, CreateUser}, ApiV1State, services::users::UserPDO};



fn filter_user_record(user: &User) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        role: user.role.to_owned(),
        verified: user.verified,
    }
}

pub async fn register_user_handler(
    State(data): State<Arc<ApiV1State<'_>>>,
    Json(user): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    

    let user = UserPDO::register_user(&data, &RegisterUserSchema {
        name: user.name.clone(),
        email: user.email.clone(),
        password: user.password.clone(),
    }, None).await.map_err(|err| {
        let error_response = serde_json::json!( {
            "status": "fail",
            "message": err.to_string(),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "user": filter_user_record(&user)
    })});

    Ok(Json(user_response))
}

pub async fn encode_token(user_id: Uuid, password_rev: Uuid, jwt_secret: &str) -> anyhow::Result<String> {
    let claims: JwtUser = JwtUser {
        id: user_id.to_string(),
        password_rev: password_rev.to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )?;

    Ok(token)
}

pub async fn login_user_handler(
    State(data): State<Arc<ApiV1State<'_>>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {


    let user = UserPDO::login_user(&data, &body).await.map_err(|err| {
        let error_response = serde_json::json!( {
            "status": "fail",
            "message": err.to_string(),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;
    
    let token = encode_token(user.id, user.password_rev, data.env.jwt_secret.as_ref())
        .await
        .map_err(|e| {
            let error_response = serde_json::json!( {
                "status": "fail",
                "message": e.to_string(),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::hours(12))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    let mut response = Response::new(json!({"status": "success", "token": token}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

pub async fn logout_handler() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    let mut response = Response::new(json!({"status": "success"}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

pub async fn get_me_handler(
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "user": filter_user_record(&user)
        })
    });

    Ok(Json(json_response))
}

pub async fn get_users_handler() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "users": ""
        })
    });

    Ok(Json(json_response))
}

// update password
pub async fn update_password(
    State(data): State<Arc<ApiV1State<'_>>>,
    Extension(user): Extension<User>,
    Json(body): Json<UpdatePasswordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    UserPDO::update_user_password(&data, &user.id.to_string(), &body.password).await.map_err(|err| {
        let error_response = serde_json::json!( {
            "status": "fail",
            "message": err.to_string(),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user_response = serde_json::json!({"status": "success","data": ()});

    Ok(Json(user_response))
}

// update user
pub async fn update_user(
    State(data): State<Arc<ApiV1State<'_>>>,
    Extension(user): Extension<User>,
    Json(body): Json<UpdateUser>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    UserPDO::update_user(&data, &UpdateUserData {
        name: body.name.clone(),
        email: body.email.clone(),
        role: user.role.clone(),
        verified: user.verified,
        id: user.id.to_string(),
    }).await.map_err(|err| {
        let error_response = serde_json::json!( {
            "status": "fail",
            "message": err.to_string(),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user_response = serde_json::json!({"status": "success","data": ()});

    Ok(Json(user_response))
}

// force update user
pub async fn force_update_user(
    State(data): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    Json(body): Json<ForceUpdateUser>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    UserPDO::update_user(&data, &UpdateUserData {
        name: body.name.clone(),
        email: body.email.clone(),
        role: body.role.clone(),
        verified: body.verified,
        id: body.id.clone(),
    }).await.map_err(|err| {
        let error_response = serde_json::json!( {
            "status": "fail",
            "message": err.to_string(),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user_response = serde_json::json!({"status": "success","data": ()});

    Ok(Json(user_response))
}

// create user

pub async fn create_user(
    State(data): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    Json(body): Json<CreateUser>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let user = UserPDO::create_user(&data, &CreateUser {
        name: body.name.clone(),
        email: body.email.clone(),
        password: body.password.clone(),
        role: body.role.to_string(),
        verified: body.verified
    }).await.map_err(|err| {
        let error_response = serde_json::json!( {
            "status": "fail",
            "message": err.to_string(),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "user": filter_user_record(&user)
    })});

    Ok(Json(user_response))
}

// delete user
pub async fn delete_user(
    State(data): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    UserPDO::delete_user(&data, &user_id).await.map_err(|err| {
        let error_response = serde_json::json!( {
            "status": "fail",
            "message": err.to_string(),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user_response = serde_json::json!({"status": "success","data": ()});

    Ok(Json(user_response))
}


