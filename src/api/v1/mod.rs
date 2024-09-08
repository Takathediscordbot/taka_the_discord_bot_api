pub mod models;
pub mod middlewares;
pub mod controllers;
pub mod services;

use std::{path::PathBuf, sync::Arc, time::Duration};
use common::LeagueRecordRequest;
use controllers::silly_command_controller::get_commands;
use headless_chrome::{protocol::cdp::{CacheStorage::DataEntry, Page::CaptureScreenshotFormatOption}, LaunchOptions};

use axum::{Router, response::IntoResponse, routing::{get, post}, extract::{State, Path, Query}, Json};
use headless_chrome::Browser;
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use serde_json::json;
#[cfg(feature = "database")]
use sqlx::{postgres::PgPoolOptions, PgPool};
use tetrio_api::{http::{ cached_client::CachedClient, caches::moka::MokaCache, clients::reqwest_client::{RedisReqwestClient, ReqwestClient}, parameters::{personal_user_records::{PersonalLeaderboard, PersonalRecordsQuery}, value_bound_query::{Prisecter, ValueBoundQuery}}}, models::{cache::Cache, packet::SuccessPacket}};
#[cfg(feature = "tetrio")]
use tetrio_api::models::packet::Packet;
use tower_http::services::ServeDir;




use super::Error;


// use crate::api::{api_v1::{middlewares::auth::{auth, is_admin}, controllers::{silly_command_controller::get_commands, user_controller::{register_user_handler, login_user_handler, logout_handler, get_me_handler, get_users_handler, update_password, update_user, force_update_user, create_user, delete_user, encode_token}}, services::users}, Error};

// use self::{services::{silly_command::SillyCommandPDO, users::UserPDO}, models::user::{FilteredUser, User, RegisterUserSchema, JwtUser, LoginUserSchema, UpdatePasswordSchema, UpdateUserData, UpdateUser, ForceUpdateUser, CreateUser}};

type TetoResponse = Packet<Box<[u8]>>;

#[derive(Serialize)]
struct TetraData {
    replay_id: Option<String>, 
    buffer: Box<[u8]>
}

#[derive(Deserialize)]
struct TetraQuery {
    pub user_id: String,
    pub game_num: u32
}


#[derive(Deserialize)]
struct FullLeaderboardQuery {
    pub country: Option<String>,
}

type TetraResponse = Packet<TetraData>;

#[allow(dead_code)]
struct Env {
    jwt_secret: String
}

#[allow(dead_code)]
pub struct ApiV1State<'a> {
    // sql_connection: PgPool,
    #[cfg(feature = "tetrio")]
    http_client: tetrio_api::http::clients::reqwest_client::RedisReqwestClient<'a>,

    #[cfg(feature = "tetrio")]
    html_server_url: String,
    #[cfg(feature = "database")]
    sql_connection: PgPool,
    env: Env
}

pub fn create_browser(width: u32, height: u32) -> Result<Browser, Error> {

    let launch_options = LaunchOptions::default_builder()
    .headless(true)
    // .path(Some("/var/www/taka_the_discord_bot/headless-chrome/chrome-linux/chrome".into()))
    // // .fetcher_options(FetcherOptions::default().with_revision(browser_version))
    .window_size(Some((width, height)))
    .sandbox(false)
    .idle_browser_timeout(Duration::from_secs(15))
    .build().map_err(|e| 
        Error(format!("Couldn't create browser build options! {e}"))
    )?;

    log::debug!("made browser configuration");

    let browser = headless_chrome::Browser::new(launch_options).map_err(|e| 
        Error(format!("Couldn't create browser! {e}"))
    )?;
    log::debug!("launched browser");

    Ok(browser)
}


pub async fn api_v1() -> Result<Router<()>, Error>{

    create_browser(200, 200)?;



    // let sql_connection_url =
    //     &std::env::var("DATABASE_URL").map_err(|e| Error(format!("Couldn't get DATABASE_URL env variable! {e}")))?;

    // let sql_connection = PgPoolOptions::new()
    //     .max_connections(25)
    //     .connect(sql_connection_url)
    //     .await.map_err(|e| Error(format!("Couldn't initialize connection pool! {e}")))?;

    // let row: (i64,) = sqlx::query_as("SELECT $1")
    //     .bind(150_i64)
    //     .fetch_one(&sql_connection)
    //     .await.map_err(|e| Error(format!("Couldn't query from database! {e}")))?;

    // log::info!("Database working ! {row:?}");

    let html_server_url = std::env::var("HTML_SERVER_URL").map_err(|e|
       Error(format!("Couldn't get env variable HTML_SERVER_URL {e}"))
    )?;

    #[cfg(feature = "database")]

    let sql_connection_url =
        &std::env::var("DATABASE_URL").expect("Couldn't get DATABASE_URL");
    #[cfg(feature = "database")]
    let sql_connection = PgPoolOptions::new()
        .max_connections(25)
        .connect(sql_connection_url)
        .await.expect("Couldn't initialize connection pool");

    let redis_url = std::env::var("REDIS_URL").expect("Couldn't get tetrio token");
    let client = redis::Client::open(redis_url).expect("Couldn't open redis connection");

    let state = Arc::new(ApiV1State{
        // sql_connection,
        http_client: RedisReqwestClient::new(
            ReqwestClient::default(),
            tetrio_api::http::caches::redis_cache::RedisCache { client: std::borrow::Cow::Owned(client) }
        ),
        html_server_url,
        env: Env {
            jwt_secret: std::env::var("JWT_SECRET").map_err(|e|
                Error(format!("Couldn't get env variable JWT_SECRET {e}"))
            )?
        },
        sql_connection
    });

    // let user = users::UserPDO::fetch_user_by_id(&state, "650caddd-b045-43d5-b691-dcc749e24b3c").await.expect("Couldn't find admin user").expect("Couldn't find admin user");
    // let token = encode_token(user.id, user.password_rev, state.env.jwt_secret.as_ref()).await.expect("Couldn't encode admin user token");
    // eprintln!("Bot token: {token}");

    let api = 
    Router::new()
        .route("/", get(hello))
        .route("/teto/:user", get(teto))
        .route("/tetra", get(tetra))
        .route("/tetra/replay", post(tetra_replay))
        .route("/league_recent_test", get(league_recent_test))        
        .route("/get_commands", get(get_commands))
        .route("/full_leaderboard", get(full_leaderboard))
        .nest_service("/images", ServeDir::new(PathBuf::from("assets")))
        .with_state(Arc::clone(&state));
        // .route("/auth/register", post(register_user_handler))
        // .route("/auth/login", post(login_user_handler))
        // .route(
        //     "/auth/logout",
        //     get(logout_handler)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth)),
        // )
        // .route(
        //     "/users/me",
        //     get(get_me_handler)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth)),
        // )
        // .route("/users",
        //     get(get_users_handler)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // .route("/users/update_password",
        //     post(update_password)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        // )
        // .route("/users/update_user",
        //     post(update_user)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        // )
        // .route("/users/force_update_user",
        //     post(force_update_user)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // .route("/users/create_user",
        //     post(create_user)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // .route("/users/delete_user/:user_id",
        //     get(delete_user)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // .route("/silly_commands/add_image",
        //     post(controllers::silly_command_controller::add_image)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // .route("/silly_commands/add_image_author",
        //     post(controllers::silly_command_controller::add_image_author)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // .route("/silly_commands/create_command",
        //     post(controllers::silly_command_controller::create_command)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // .route("/silly_commands/add_preference",
        //     post(controllers::silly_command_controller::add_preference)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )

        // // text
        // .route("/silly_commands/add_text",
        //     post(controllers::silly_command_controller::add_text)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // //text author
        // .route("/silly_commands/add_text_author",
        //     post(controllers::silly_command_controller::add_text_author)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // .route("/silly_commands/delete_preference",
        //     post(controllers::silly_command_controller::delete_preference)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // .route("/silly_commands/delete_command",
        //     post(controllers::silly_command_controller::delete_command)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // // update
        // .route("/silly_commands/update_command",
        //     post(controllers::silly_command_controller::update_command)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // // update image
        // .route("/silly_commands/update_image",
        //     post(controllers::silly_command_controller::update_image)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // // update image author
        // .route("/silly_commands/update_image_author",
        //     post(controllers::silly_command_controller::update_image_author)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // // update text
        // .route("/silly_commands/update_text",
        //     post(controllers::silly_command_controller::update_text)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // )
        // // update text author
        // .route("/silly_commands/update_text_author",
        //     post(controllers::silly_command_controller::update_text_author)
        //         .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        //         .route_layer(middleware::from_fn(is_admin))
        // );

    Ok(api)
}


async fn hello() -> impl IntoResponse {
    "Hello!"
}

pub async fn get_full_leaderboard(state: &ApiV1State<'_>, country: Option<String>) -> Result<Packet<Vec<serde_json::Value>>, Box<dyn std::error::Error>> {
    let client = &state.http_client;
    let url = format!("/BEAN_BLASTER_SERVER?country={country:?}");
    let session_id = "X_TEST_BEAN_BLASTER";
    match client.get_from_cache::<Vec<serde_json::Value>>(&url, Some(&session_id)).await? {
        Some(packet) => return Ok(packet),
        None => {}
    };

    let mut query = ValueBoundQuery::NotBound { limit: Some(100), country: country.clone(), };
    let url = format!("users/by/{}", "league");
    let result = client.make_tetrio_api_request::<serde_json::Value>(CachedClient::<ReqwestClient, MokaCache>::make_url(&url, &query.as_query_params()), Some(session_id)).await?;
    let result = result.data.unwrap_or(json! ({ 
      "entries": []
    }));
    let default_p = json!( Prisecter {pri: 0., sec: 0., ter: 0. } );
    let entries = result.get("entries").ok_or("Couldn't fetch leaderboard!")?.as_array().ok_or("Couldn't fetch leaderboard!")?;
    let p = entries.last().map(|entry| serde_json::from_value::<Prisecter>(entry.get("p").unwrap_or(&default_p).clone())).ok_or("Couldn't fetch leaderboard!")??;

    let mut results = vec![];
    results.push(entries.clone());

    query = ValueBoundQuery::After { after: p, limit: Some(100), country: country.clone() };
    loop {
      
      let session_id = "X_BEANBLASTER";
      let url = format!("users/by/{}", "league");
      let result = client.make_tetrio_api_request::<serde_json::Value>(CachedClient::<ReqwestClient, MokaCache>::make_url(&url, &query.as_query_params()), Some(session_id)).await?;
      let result = result.data.unwrap_or(json! ({ 
        "entries": []
      }));
      let default_p = json!( Prisecter {pri: 0., sec: 0., ter: 0. } );
      let entries = result.get("entries").ok_or("Couldn't fetch leaderboard!")?.as_array().ok_or("Couldn't fetch leaderboard!")?;
      
      let p = entries.last().map(|entry| serde_json::from_value::<Prisecter>(entry.get("p").unwrap_or(&default_p).clone())).ok_or("Couldn't fetch leaderboard!")??;
     
      query = ValueBoundQuery::After { after: p, limit: Some(100), country: country.clone() };
  
      results.push(entries.clone());


      if entries.len() != 100 {
        break;
      }
    };


    let result = Packet {
        success: true,
        error: None,
        data: Some(results.into_iter().flatten().collect_vec()),
        cache: Some(Cache::cached_for(Duration::from_secs(3600)))
    };

    state.http_client.cache_tetrio_api_result_if_not_present::<serde_json::Value>(url, Some(&session_id), serde_json::to_value(&result)?).await?;

    Ok(result)
    
}

pub async fn full_leaderboard(State(state): State<Arc<ApiV1State<'_>>>, Query(query): Query<FullLeaderboardQuery>) -> impl IntoResponse {
    let leaderboard = get_full_leaderboard(&state, query.country).await;


    Json(
    match leaderboard {
        Err(err) => Packet {
            cache: None,
            data: None,
            error: Some(tetrio_api::models::packet::Error { msg: err.to_string() }),
            success: false,
        },
        Ok(result) => result
    }).into_response()
}

#[derive(Deserialize)]
pub struct TetraTestParam {
    left_score: Option<u32>,
    right_score: Option<u32>,
}

async fn take_tetra_league_screenshot_of_url(rounds: u64, url: String) -> Result<Vec<u8>, Error> {
    log::debug!("made configuration");

        let browser = create_browser(
            1185,
            350 + 60 * (rounds - 1) as u32)?;

        log::debug!("launched browser");
        let tab = browser.new_tab().map_err(|e| Error(format!("Couldn't create new tab! {e}")))?;
        log::debug!("opened tab");

        tab.navigate_to(&url).map_err(|e| Error(format!("Couldn't load tetra league replay page! {e}")))?;
        log::debug!("navigated to tab");

        let _element = tab.wait_for_element("#multilog").map_err(|e| Error(format!("Couldn't find element to screenshot! {e}")))?;
        log::debug!("waited for element");
        let buffer =
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true).map_err(|e| Error(format!("Couldn't take screenshot! {e}")))?;
        log::debug!("took screenshot");

        tab.close(true).map_err(|e| Error(format!("Couldn't close tab {e}")))?;
        Ok(buffer)
}

async fn take_tetra_league_test_screenshot(state: &ApiV1State<'_>, left_score: Option<u32>, right_score: Option<u32>) -> Result<TetraData, Error> {
    let buffer = {
        let left_score = left_score.unwrap_or(5);
        let right_score = right_score.unwrap_or(5);
        let max_score = std::cmp::max(left_score, right_score);
        log::debug!("made configuration");

        take_tetra_league_screenshot_of_url(max_score.into(), format!(
            "{}/league_recent_test?left_score={}&right_score={}",
            state.html_server_url, left_score, right_score 
        )).await?
    };

    Ok(TetraData {
        replay_id: None,
        buffer: buffer.into_boxed_slice()
    })
}

async fn take_tetra_replay_screenshot(state: &ApiV1State<'_>, data: common::LeagueRecordRequest) -> Result<TetraData, Error> {
    if data.league_record.rounds.len() > 14 {
        return Err(Error("Replay has more than 14 rounds".to_string()));
    }

    let Ok(obj_string) = serde_json::to_string(&data) else {
        return Err(Error("Couldn't serialize data".to_string()));
    };

    let encoded = urlencoding::encode(&obj_string);

    let buffer = {
        take_tetra_league_screenshot_of_url(data.league_record.rounds.len() as u64, format!("{}/league_replay_from_data?data={}", state.html_server_url, encoded)).await?        
    };

    Ok(TetraData {
        replay_id: None,
        buffer: buffer.into_boxed_slice()
    })

}


async fn take_tetra_screenshot(state: &ApiV1State<'_>, user: &str, game_num: u32) -> Result<TetraData, Error> {
    let packet = state.http_client
        .fetch_user_personal_league_records(&user, PersonalLeaderboard::Recent, PersonalRecordsQuery::None)
        .await
        .map_err(|e| Error(format!("Couldn't fetch tetra league game: {e}")))?;

    let Some(data) = &packet.data else {
        return Err(Error("User does not have tetra league records".to_string()))
    };

    let game_num = if game_num <= 0 { 1 } else { game_num };

    let Some(record) = data.entries.get((game_num - 1) as usize) else {
        return Err(Error("Tetra league game not found".to_string()))
    };

    let buffer = {
        let buffer = {
            take_tetra_league_screenshot_of_url(record.results.rounds.len() as u64, format!(
                "{}/league_replay?user_id={}&replay_id={}",
                state.html_server_url, user, record.replayid
            )).await?        
        };
    
        buffer
    };

    Ok(TetraData {
        replay_id: None,
        buffer: buffer.into_boxed_slice()
    })
}

async fn tetra_replay(State(state): State<Arc<ApiV1State<'_>>>,
    Json(payload): axum::extract::Json<LeagueRecordRequest>) -> impl IntoResponse {

    match take_tetra_replay_screenshot(&state, payload).await {    
        Ok(data) => Json(TetraResponse {
            success: true,
            data: Some(data),
            cache: None,
            error: None
        }).into_response(),
        Err(err) => Json(TetraResponse {
            success: false,
            data: None,
            cache: None,
            error: Some(tetrio_api::models::packet::Error {msg: err.0 })
        }).into_response()
    }
 }

async fn tetra(State(state): State<Arc<ApiV1State<'_>>>,
 Query(query): Query<TetraQuery>) -> impl IntoResponse {
    match take_tetra_screenshot(&state, &query.user_id, query.game_num).await {    
        Ok(data) => Json(TetraResponse {
            success: true,
            data: Some(data),
            cache: None,
            error: None
        }).into_response(),
        Err(err) => Json(TetraResponse {
            success: false,
            data: None,
            cache: None,
            error: Some(tetrio_api::models::packet::Error {msg: err.0 })
        }).into_response()
    }

    
}

async fn league_recent_test(State(state): State<Arc<ApiV1State<'_>>>,
 Query(query): Query<TetraTestParam>) -> impl IntoResponse {
    let TetraTestParam { left_score, right_score } = query;
    match take_tetra_league_test_screenshot(&state, left_score, right_score).await {    
        Ok(data) => Json(TetraResponse {
            success: true,
            data: Some(data),
            cache: None,
            error: None
        }).into_response(),
        Err(err) => Json(TetraResponse {
            success: true,
            data: None,
            cache: None,
            error: Some(tetrio_api::models::packet::Error {msg: err.0 })
        }).into_response()
    }
}

async fn take_teto_screenshot(state: &ApiV1State<'_>, user: &str) -> Result<Vec<u8>, Error> {
    log::debug!("made configuration");

    let browser = create_browser(900, 500).map_err(|e| Error(format!("Couldn't create browser ! {e}")))?;

    let tab = browser.new_tab().map_err(|e| Error(format!("Couldn't create new tab ! {e}")))?;

    tab.set_transparent_background_color().map_err(|e| Error(format!("Couldn't set transparent background ! {e}")))?;

    tab.navigate_to(&format!(
        "{}/teto_test/{}",
        state.html_server_url,
        user.to_lowercase()
    )).map_err(|e| Error(format!("Couldn't navigate to url ! {e}")))?;
    log::debug!("navigated to tab");

    tab.wait_until_navigated().map_err(|e| Error(format!("Couldn't wait for tab to finish navigating! {e}")))?;

    tokio::time::sleep(Duration::from_millis(750)).await;

    let element = tab.wait_for_element(".tetra_modal").map_err(|e| Error(format!("Couldn't find element to screenshot! {e}")))?;
    log::debug!("waited for element");

    let viewport = element.get_box_model().map_err(|e| Error(format!("Couldn't find size of element ! {e}")))?;
    let mut viewport = viewport.border_viewport();
    viewport.x -= 16.0;
    viewport.y -= 16.0;
    viewport.width += 32.0;
    viewport.height += 32.0;

    let buffer = tab.capture_screenshot(
        CaptureScreenshotFormatOption::Png,
        None,
        Some(viewport),
        true,
    ).map_err(|e| Error(format!("Couldn't take screenshot ! {e}")))?;
    log::debug!("took screenshot");
    Ok(buffer)
}


async fn teto(State(state): State<Arc<ApiV1State<'_>>>, Path(user): Path<String>) -> impl IntoResponse {
    let username = &user;
    let url = format!("teto_image_cache/{user}");
    if let Ok(Some(entry)) = state.http_client.get_from_cache::<Box<[u8]>>(&url, None).await {
        return Json(entry).into_response()
    };

    let user = match state.http_client.fetch_user_info(&username).await {
        Ok(user) => user,
        Err(err) => return Json(TetoResponse { 
            cache: None,
            data: None,
            error: Some(tetrio_api::models::packet::Error {msg: format!("Couldn't fetch user from tetrio API! {err}")}),
            success: false
        }).into_response()
    };

    if let None = user.data {
        return Json(TetoResponse { 
            cache: None,
            data: None,
            error: Some(tetrio_api::models::packet::Error {msg: format!("Couldn't find user")}),
            success: false
        }).into_response();
    }




    let buffer = match take_teto_screenshot(&state, &username).await {
        Ok(ok) => ok, 
        Err(err) => return Json(TetoResponse { 
            cache: None,
            data: None,
            error: Some(tetrio_api::models::packet::Error {msg: format!("Couldn't take teto screenshot {err}")}),
            
            success: false
        }).into_response()
    };


    let Some(cache) = &user.cache else {
        return Json(TetoResponse { 
            cache: None,
            data: None,
            error: Some(tetrio_api::models::packet::Error {msg: format!("Couldn't find user")}),
            success: false
        }).into_response();
    };
    
    let entry = SuccessPacket {
    
        data: Some(buffer.into_boxed_slice()),
    
        cache: cache.clone(),
    
        success: true
    
    };

    let Ok(json) = serde_json::to_value(entry) else {
        return Json(TetoResponse { 
            cache: None,
            data: None,
            error: Some(tetrio_api::models::packet::Error {msg: format!("Couldn't parse to json!")}),
            success: false
        }).into_response();
    };
    
    let Ok(entry) = state.http_client.cache_tetrio_api_result_if_not_present::<serde_json::Value>(url, None, json).await else {
        return Json(TetoResponse { 
            cache: None,
            data: None,
            error: Some(tetrio_api::models::packet::Error {msg: format!("Couldn't cache value!")}),
            success: false
        }).into_response();
    };

    return Json(entry).into_response()
        
    

}


