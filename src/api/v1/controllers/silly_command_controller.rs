#![cfg(feature = "database")]

use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json, Router,
};
use serde_json::json;
use tower_http::services::ServeDir;

use crate::api::{api_v1::{
    models::{silly_command::{AddCommandRequest, AddPreferenceRequest, AddTextAuthorRequest, AddTextRequest, FetchRandomSillyImageByNameAndPreference, FetchSillyCommandByName}, user::User}, services::silly_command::SillyCommandPDO, ApiV1State,
}, v1::models::silly_command::SillyCommandData};

pub fn router() -> Router {
    return Router::new()
        .nest_service("/images", ServeDir::new(PathBuf::from("assets")))

}

pub async fn get_commands(State(state): State<Arc<ApiV1State<'_>>>) -> impl IntoResponse {
    Json::<Vec<SillyCommandData>>(SillyCommandPDO::fetch_silly_commands(&state).await).into_response()
}



/*
.route("/silly_commands/add_image",
            post(controllers::silly_command_controller::add_image)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        .route("/silly_commands/add_image_author",
            post(controllers::silly_command_controller::add_image_author)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        .route("/silly_commands/create_command",
            post(controllers::silly_command_controller::create_command)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        .route("/silly_commands/add_preference",
            post(controllers::silly_command_controller::add_preference)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        .route("/silly_commands/delete_preference",
            post(controllers::silly_command_controller::delete_preference)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        .route("/silly_commands/delete_command",
            post(controllers::silly_command_controller::delete_command)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        // text
        .route("/silly_commands/add_text",
            post(controllers::silly_command_controller::add_text)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        // text author
        .route("/silly_commands/add_text_author",
            post(controllers::silly_command_controller::add_text_author)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        // update
        .route("/silly_commands/update_command",
            post(controllers::silly_command_controller::update_command)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        // update image
        .route("/silly_commands/update_image",
            post(controllers::silly_command_controller::update_image)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        // update image author
        .route("/silly_commands/update_image_author",
            post(controllers::silly_command_controller::update_image_author)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        // update text
        .route("/silly_commands/update_text",
            post(controllers::silly_command_controller::update_text)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        // update text author
        .route("/silly_commands/update_text_author",
            post(controllers::silly_command_controller::update_text_author)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth))
                .route_layer(middleware::from_fn(is_admin))
        )
        .with_state(state);
*/

struct AddImageRequest {
    pub command_name: Option<String>,
    pub image: Option<Vec<u8>>,
    pub extension: Option<String>,
    pub preference: Option<String>,
}

pub async fn add_image(
    State(state): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    mut body: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Extract image, command name, command type, and call SillyCommandPDO::add_image
    // return the id of the image
    let mut req = AddImageRequest {
        command_name: None,
        image: None,
        extension: None,
        preference: None,
    };
    while let Some(part) = body.next_field().await.map_err(|e| {
        let json_error = json!({
            "status": "fail",
            "message": format!("Internal server error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
    })? {
        match part.name() {
            None => {
                let json_error = json!({
                    "status": "fail",
                    "message": "Invalid field name",
                });
                return Err((StatusCode::BAD_REQUEST, Json(json_error)));
            }
            Some(data) => {
                match data {
                    "image" => {
                        let image = part.bytes().await.map_err(|e| {
                            let json_error = json!({
                                "status": "fail",
                                "message": format!("Internal server error: {}", e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
                        })?;

                        req.image = Some(image.to_vec());

                        // let image_id = SillyCommandPDO::add_image(&state, &image).await.map_err(|e| {
                        //     let json_error = json!({
                        //         "status": "fail",
                        //         "message": format!("Internal server error: {}", e),
                        //     });
                        //     (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
                        // })?;
                    }
                    "command_name" => {
                        let command_name = part.text().await.map_err(|e| {
                            let json_error = json!({
                                "status": "fail",
                                "message": format!("Internal server error: {}", e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
                        })?;

                        let command_name = command_name.to_string();

                        let command_id =
                            SillyCommandPDO::fetch_silly_command_by_name(&state, &command_name)
                                .await;

                        if command_id.is_none() {
                            let json_error = json!({
                                "status": "fail",
                                "message": "Invalid command name",
                            });
                            return Err((StatusCode::BAD_REQUEST, Json(json_error)));
                        }

                        req.command_name = Some(command_name);
                    }
                    "extension" => {
                        let extension = part.text().await.map_err(|e| {
                            let json_error = json!({
                                "status": "fail",
                                "message": format!("Internal server error: {}", e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
                        })?;

                        req.extension = Some(extension.to_string());
                    }
                    "preference" => {
                        let preference = part.text().await.map_err(|e| {
                            let json_error = json!({
                                "status": "fail",
                                "message": format!("Internal server error: {}", e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
                        })?;

                        req.preference = Some(preference.to_string());
                    }
                    _ => {
                        let json_error = json!({
                            "status": "fail",
                            "message": "Invalid field name",
                        });
                        return Err((StatusCode::BAD_REQUEST, Json(json_error)));
                    }
                }
            }
        }
    }

    let AddImageRequest {
        command_name: Some(command_name),
        image: Some(image),
        extension: Some(extension),
        preference,
    } = req
    else {
        let json_error = json!({
            "status": "fail",
            "message": "Invalid request",
        });
        return Err((StatusCode::BAD_REQUEST, Json(json_error)));
    };

    let image_id = SillyCommandPDO::add_image(&state, &command_name, image, &extension, preference)
        .await
        .map_err(|e| {
            let json_error = json!({
                "status": "fail",
                "message": format!("Internal server error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let image_response = json!({
        "status": "success",
        "data": {
            "image_id": image_id,
        }
    });

    Ok(Json(image_response))
}

struct AddImageAuthorRequest {
    pub command_name: Option<String>,
    pub image: Option<Vec<u8>>,
    pub extension: Option<String>,
}

// add image author
pub async fn add_image_author(
    State(state): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    mut body: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Extract image, command name, command type, and call SillyCommandPDO::add_image
    // return the id of the image
    let mut req = AddImageAuthorRequest {
        command_name: None,
        image: None,
        extension: None,
    };
    while let Some(part) = body.next_field().await.map_err(|e| {
        let json_error = json!({
            "status": "fail",
            "message": format!("Internal server error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
    })? {
        match part.name() {
            None => {
                let json_error = json!({
                    "status": "fail",
                    "message": "Invalid field name",
                });
                return Err((StatusCode::BAD_REQUEST, Json(json_error)));
            }
            Some(data) => {
                match data {
                    "image" => {
                        let image = part.bytes().await.map_err(|e| {
                            let json_error = json!({
                                "status": "fail",
                                "message": format!("Internal server error: {}", e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
                        })?;

                        req.image = Some(image.to_vec());

                        // let image_id = SillyCommandPDO::add_image(&state, &image).await.map_err(|e| {
                        //     let json_error = json!({
                        //         "status": "fail",
                        //         "message": format!("Internal server error: {}", e),
                        //     });
                        //     (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
                        // })?;
                    }
                    "command_name" => {
                        let command_name = part.text().await.map_err(|e| {
                            let json_error = json!({
                                "status": "fail",
                                "message": format!("Internal server error: {}", e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
                        })?;

                        let command_name = command_name.to_string();

                        let command_id =
                            SillyCommandPDO::fetch_silly_command_by_name(&state, &command_name)
                                .await;

                        if command_id.is_none() {
                            let json_error = json!({
                                "status": "fail",
                                "message": "Invalid command name",
                            });
                            return Err((StatusCode::BAD_REQUEST, Json(json_error)));
                        }

                        req.command_name = Some(command_name);
                    }
                    "extension" => {
                        let extension = part.text().await.map_err(|e| {
                            let json_error = json!({
                                "status": "fail",
                                "message": format!("Internal server error: {}", e),
                            });
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
                        })?;

                        req.extension = Some(extension.to_string());
                    }

                    _ => {
                        let json_error = json!({
                            "status": "fail",
                            "message": "Invalid field name",
                        });
                        return Err((StatusCode::BAD_REQUEST, Json(json_error)));
                    }
                }
            }
        }
    }

    let AddImageAuthorRequest {
        command_name: Some(command_name),
        image: Some(image),
        extension: Some(extension),
    } = req
    else {
        let json_error = json!({
            "status": "fail",
            "message": "Invalid request",
        });
        return Err((StatusCode::BAD_REQUEST, Json(json_error)));
    };

    let image_id = SillyCommandPDO::add_image_author(&state, &command_name, image, &extension)
        .await
        .map_err(|e| {
            let json_error = json!({
                "status": "fail",
                "message": format!("Internal server error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let image_response = json!({
        "status": "success",
        "data": {
            "image_id": image_id,
        }
    });

    Ok(Json(image_response))
}

// JSON(AddCommandRequest)

pub async fn create_command(
    State(state): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    Json(body): Json<AddCommandRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Extract command name, command type, and call SillyCommandPDO::add_command
    // return the id of the command
    let command_id = SillyCommandPDO::create_command(&state, &body.command_name, &body.description, &body.footer_text, body.command_type)
        .await
        .map_err(|e| {
            let json_error = json!({
                "status": "fail",
                "message": format!("Internal server error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let command_response = json!({
        "status": "success",
        "data": {
            "command_id": command_id,
        }
    });

    Ok(Json(command_response))
}

// JSON(AddTextRequest)
pub async fn add_text(
    State(state): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    Json(body): Json<AddTextRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Extract command name, command type, and call SillyCommandPDO::add_text
    // return the id of the text
    let text_id = SillyCommandPDO::add_text(&state, &body.command_name, &body.content)
        .await
        .map_err(|e| {
            let json_error = json!({
                "status": "fail",
                "message": format!("Internal server error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let text_response = json!({
        "status": "success",
        "data": {
            "text_id": text_id,
        }
    });

    Ok(Json(text_response))
}

// JSON(AddTextAuthorRequest)
pub async fn add_text_author(
    State(state): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    Json(body): Json<AddTextAuthorRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Extract command name, command type, and call SillyCommandPDO::add_text
    // return the id of the text
    let text_id = SillyCommandPDO::add_text_author(&state, &body.command_name, &body.content)
        .await
        .map_err(|e| {
            let json_error = json!({
                "status": "fail",
                "message": format!("Internal server error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let text_response = json!({
        "status": "success",
        "data": {
            "text_id": text_id,
        }
    });

    Ok(Json(text_response))
}

// JSON(AddPreferenceRequest)
pub async fn add_preference(
    State(state): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    Json(body): Json<AddPreferenceRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Extract command name, command type, and call SillyCommandPDO::add_preference
    // return the id of the preference
    let preference_id = SillyCommandPDO::add_preference(&state, &body.command_name, &body.preference)
        .await
        .map_err(|e| {
            let json_error = json!({
                "status": "fail",
                "message": format!("Internal server error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let preference_response = json!({
        "status": "success",
        "data": {
            "preference_id": preference_id,
        }
    });

    Ok(Json(preference_response))
}


// JSON(FetchRandomSillyImageByNameAndPreference)
pub async fn fetch_random_silly_image_by_name_and_preference(
    State(state): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    Json(body): Json<FetchRandomSillyImageByNameAndPreference>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Extract command name, command type, and call SillyCommandPDO::fetch_random_silly_image_by_name_and_preference
    // return the id of the preference
    let image = SillyCommandPDO::fetch_random_silly_image_by_name_and_preference(&state, body.command, &body.preference)
        .await
        .map_err(|e| {
            let json_error = json!({
                "status": "fail",
                "message": format!("Internal server error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let image_response = json!({
        "status": "success",
        "data": {
            "image": image,
        }
    });

    Ok(Json(image_response))

}

// JSON(FetchSillyCommandByName)
pub async fn fetch_silly_command_by_name(
    State(state): State<Arc<ApiV1State<'_>>>,
    Extension(_): Extension<User>,
    Json(body): Json<FetchSillyCommandByName>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Extract command name, command type, and call SillyCommandPDO::fetch_random_silly_image_by_name_and_preference
    // return the id of the preference
    let command = SillyCommandPDO::fetch_silly_command_by_name(&state, &body.name)
        .await
        .ok_or_else(|| {
            let json_error = json!({
                "status": "fail",
                "message": "Couldn't find command",
            });
            (StatusCode::NOT_FOUND, Json(json_error))
        })?;

    let command_response = json!({
        "status": "success",
        "data": {
            "command": command,
        }
    });

    Ok(Json(command_response))

}