mod cfg;
mod matrix;

use std::sync::Arc;

use axum::{debug_handler, extract::State, http::StatusCode, routing::{get, post}, Form};
use cfg::Config;
use eyre::{Context, Result};
use matrix_sdk::{config::SyncSettings, ruma::{events::room::message::RoomMessageEventContent, RoomAliasId, UserId}, Client as MatrixClient};
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<()> {
    let config = cfg::Config::create()?;
    let client = matrix::init_matrix_client(&config).await?;
    let state = Arc::new(AppState { client: client, cfg: config });
    if let Err(e) = tokio::try_join!(sync(&state), run_server(state.clone())) {
        println!("abnormal termination: {}", e);
    }

    Ok(())
}

#[derive(Clone)]
struct AppState {
    client: MatrixClient,
    cfg: Config,
}

#[derive(Deserialize)]
struct TwilioNewMessage {
    #[serde(rename = "From")]
    from: String,
    #[serde(rename = "To")]
    to: String,
    #[serde(rename = "Body")]
    body: String,
}

async fn sync(state: &AppState) -> Result<()> {
    let user_id = UserId::parse(&state.cfg.matrix_user_id)
        .wrap_err("failed to parse matrix user id")?;
    
    println!("authenticating with username = {}", user_id);
    state.client.matrix_auth()
        .login_username(user_id, &state.cfg.matrix_password)
        .await
        .wrap_err("failed to authenticate with matrix homeserver")?;

    print!("running continuous sync");
    state.client.sync(SyncSettings::default()).await
        .wrap_err("failed to sync matrix client")
}

async fn run_server(state: Arc<AppState>) -> Result<()> {
    let app = axum::Router::new()
        .route("/health/startup", get(get_startup))
        .route("/health/liveness", get(get_liveness))
        .route("/health/readiness", get(get_readiness))
        .route("/twilio/messages", post(post_twilio_message))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9090")
        .await
        .wrap_err("failed to start tokio listener")?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_startup(
    State(_): State<Arc<AppState>>
) -> Result<(), StatusCode> {
    Ok(())
}

async fn get_liveness(
    State(_): State<Arc<AppState>>
) -> Result<(), StatusCode> {
    Ok(())
}

async fn get_readiness(
    State(state): State<Arc<AppState>>
) -> Result<(), StatusCode> {
    match state.client.logged_in() {
        true => Ok(()),
        false => Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[debug_handler]
async fn post_twilio_message(
    State(state): State<Arc<AppState>>,
    Form(data): Form<TwilioNewMessage>,
) -> Result<(), StatusCode> {
    println!("new msg: from = {}, to = {}, body = {}, room = {}", data.from, data.to, data.body, &state.cfg.matrix_room_id);

    let room_alias = match RoomAliasId::parse(&state.cfg.matrix_room_id) {
        Ok(r) => r,
        Err(e) => {
            println!("failed to parse room alias: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        },
    };
    let room_id = match state.client.resolve_room_alias(&room_alias).await {
        Ok(r) => r.room_id,
        Err(e) => {
            println!("failed to resolve room room id: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        },
    };
    let content = RoomMessageEventContent::text_plain(
        format!("FROM: {}\nTO: {}\n\n{}", data.from, data.to, data.body));
    let room = match state.client.get_room(&room_id) {
        Some(r) => r,
        None => {
            println!("room id = {} not found", room_id);
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    };

    if let Err(e) = room.send(content).await {
        println!("failed to send message to room: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    Ok(())
}

