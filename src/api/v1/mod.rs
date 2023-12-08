use std::{sync::Arc, time::Duration};
use headless_chrome::{protocol::cdp::Page::CaptureScreenshotFormatOption, LaunchOptions};

use axum::{Router, response::IntoResponse, routing::get, extract::{State, Path, Query}, Json};
use headless_chrome::Browser;
use moka::future::Cache;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tetrio_api::{http::cached_client::CachedClient, models::packet::{Packet, CacheExpiration}};

use crate::Error;

type TetoResponse = Packet<Box<[u8]>>;

#[derive(Serialize)]
struct TetraData {
    replay_id: String, 
    buffer: Box<[u8]>
}

type TetraResponse = Packet<TetraData>;

#[allow(dead_code)]
struct ApiV1State {
    sql_connection: PgPool,
    http_client: tetrio_api::http::cached_client::CachedClient,
    teto_cache: Cache<Box<str>, Arc<TetoResponse>>,
    html_server_url: String
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


pub async fn api_v1() -> Result<Router, Error> {

    create_browser(200, 200)?;

    let sql_connection_url =
        &std::env::var("DATABASE_URL").map_err(|e| Error(format!("Couldn't get DATABASE_URL env variable! {e}")))?;

    let sql_connection = PgPoolOptions::new()
        .max_connections(25)
        .connect(sql_connection_url)
        .await.map_err(|e| Error(format!("Couldn't initialize connection pool! {e}")))?;

    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&sql_connection)
        .await.map_err(|e| Error(format!("Couldn't query from database! {e}")))?;

    log::info!("Database working ! {row:?}");

    let html_server_url = std::env::var("HTML_SERVER_URL").map_err(|e|
       Error(format!("Couldn't get env variable HTML_SERVER_URL {e}"))
    )?;

    let state = Arc::new(ApiV1State{
        sql_connection,
        http_client: CachedClient::default(),
        teto_cache: Cache::builder().expire_after(CacheExpiration).build(),
        html_server_url
    });

    let api = 
    Router::new()
        .route("/", get(hello))
        .route("/teto/:user", get(teto))
        .route("/tetra", get(tetra))

        .with_state(state);

    Ok(api)
}


async fn hello() -> impl IntoResponse {
    "Hello!"
}

async fn take_tetra_screenshot(state: &ApiV1State, user: &str, game_num: u32) -> Result<TetraData, Error> {
    let packet = state.http_client
        .fetch_tetra_league_recent(&user)
        .await
        .map_err(|e| Error(format!("Couldn't fetch tetra league game: {e}")))?;

    let Some(data) = &packet.data else {
        return Err(Error("User does not have tetra league records".to_string()))
    };

    let game_num = if game_num <= 0 { 1 } else { game_num };

    let Some(record) = data.records.get((game_num - 1) as usize) else {
        return Err(Error("Tetra league game not found".to_string()))
    };

    let (Some(left), Some(_right)) = (record.endcontext.get(0), record.endcontext.get(1))
    else {
        return Err(Error("Couldn't parse tetra league data".to_string()));
    };

    let buffer = {

        log::debug!("made configuration");

        let browser = create_browser(
            1185,
            350 + 60 * (left.points.secondary_avg_tracking.len() - 1) as u32)?;

        log::debug!("launched browser");
        let tab = browser.new_tab().map_err(|e| Error(format!("Couldn't create new tab! {e}")))?;
        log::debug!("opened tab");

        tab.navigate_to(&format!(
            "{}/league_replay?user_id={}&replay_id={}",
            state.html_server_url, user, record.replay_id
        )).map_err(|e| Error(format!("Couldn't load tetra league replay page! {e}")))?;
        log::debug!("navigated to tab");

        let _element = tab.wait_for_element("#multilog").map_err(|e| Error(format!("Couldn't find element to screenshot! {e}")))?;
        log::debug!("waited for element");
        let buffer =
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true).map_err(|e| Error(format!("Couldn't take screenshot! {e}")))?;
        log::debug!("took screenshot");

        tab.close(true).map_err(|e| Error(format!("Couldn't close tab {e}")))?;
        buffer
    };

    Ok(TetraData {
        replay_id: record.replay_id.to_string(),
        buffer: buffer.into_boxed_slice()
    })
}

async fn tetra(State(state): State<Arc<ApiV1State>>,
 Query(user_id): Query<String>, Query(game_num): Query<u32>) -> impl IntoResponse {
    match take_tetra_screenshot(&state, &user_id, game_num).await {    
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
            error: Some(err.0)
        }).into_response()
    }
}

async fn take_teto_screenshot(state: &ApiV1State, user: &str) -> Result<Vec<u8>, Error> {
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

    tokio::time::sleep(Duration::from_secs(2)).await;

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


async fn teto(State(state): State<Arc<ApiV1State>>, Path(user): Path<String>) -> impl IntoResponse {
    let username = user;

    if let Some(entry) = state.teto_cache.get(&username.clone().into_boxed_str()).await {
        return Json(entry).into_response()
    };

    let user = match state.http_client.fetch_user_info(&username).await {
        Ok(user) => user,
        Err(err) => return Json(TetoResponse { 
            cache: None,
            data: None,
            error: Some(format!("Couldn't fetch user from tetrio API! {err}")),
            success: false
        }).into_response()
    };




    let buffer = match take_teto_screenshot(&state, &username).await {
        Ok(ok) => ok, 
        Err(err) => return Json(TetoResponse { 
            cache: None,
            data: None,
            error: Some(err.0),
            success: false
        }).into_response()
    };

    let entry = Arc::new(TetoResponse {
        data: Some(buffer.into_boxed_slice()),
        cache: user.cache.clone(),
        error: None,
        success: true
    });

    state.teto_cache.insert(username.into_boxed_str(), Arc::clone(&entry)).await;

    return Json(entry).into_response()
}