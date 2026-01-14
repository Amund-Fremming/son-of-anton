use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::process::Command;
use tokio::spawn;
use tracing::{error, info};

use crate::{
    app_error::AppError,
    tools::zigbee::{Brightness, ColorTemp, ZigbeeController},
};

const HTML_DIR: &str = "src/tmp_light_api";

#[derive(Clone)]
pub struct WebState {
    controller: ZigbeeController,
    password: String,
}

#[derive(Debug, Deserialize)]
pub enum LightAction {
    AllOff,
    AllOn,
    NightMode,
    MovieMode,
    PartyMode,
}

async fn start_ngrok() {
    // Read ngrok auth token from environment variable
    let ngrok_token = match std::env::var("NGROK_AUTH_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            error!("NGROK_AUTH_TOKEN environment variable not set");
            return;
        }
    };

    // Configure ngrok with auth token
    let _ = Command::new("ngrok")
        .args(&["config", "add-authtoken", &ngrok_token])
        .output();

    // Start ngrok tunnel
    info!("Starting ngrok tunnel...");
    let child = Command::new("ngrok").args(&["http", "3000"]).spawn();

    match child {
        Ok(_) => info!("Ngrok started successfully"),
        Err(e) => error!("Failed to start ngrok: {}", e),
    }
}

// Server

pub async fn start_server(controller: ZigbeeController) -> Result<(), AppError> {
    info!("Starting light controller server");

    let password = std::env::var("PASSWORD")?;
    let state = WebState {
        controller,
        password,
    };

    /*
    // Start ngrok in a separate task
    spawn(async {
        start_ngrok().await;
    });
    */

    // Build our application with routes
    let app = Router::new()
        .route("/", get(login_page))
        .route("/login/{password}", post(handle_login))
        .route("/control/{password}", get(control_page))
        .route("/set-light/{password}/{action}", post(set_light))
        .route("/style.css", get(get_css))
        .route("/script.js", get(get_js))
        .with_state(state);

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

async fn login_page() -> Html<String> {
    info!("Serving login page");
    let html = tokio::fs::read_to_string(format!("{}/login.html", HTML_DIR))
        .await
        .unwrap_or_else(|e| {
            error!("Failed to load login.html: {}", e);
            "Failed to load html".to_string()
        });

    Html(html)
}

async fn handle_login(
    State(state): State<WebState>,
    Path(password): Path<String>,
) -> impl IntoResponse {
    info!("Login attempt");
    if password != state.password {
        error!("Incorrect password");
        return (StatusCode::UNAUTHORIZED, "Nope");
    }

    info!("Successful login");
    (StatusCode::OK, "Success")
}

async fn control_page(
    State(state): State<WebState>,
    Path(password): Path<String>,
) -> impl IntoResponse {
    info!("Serving control page");
    if password != state.password {
        error!("Incorrect password");
        return (StatusCode::UNAUTHORIZED, "Nope").into_response();
    }

    let html = tokio::fs::read_to_string(format!("{}/control.html", HTML_DIR))
        .await
        .unwrap_or_else(|e| {
            error!("Failed to load control.html: {}", e);
            "Nope".to_string()
        });

    Html(html).into_response()
}

async fn set_light(
    State(state): State<WebState>,
    Path((password, action)): Path<(String, LightAction)>,
) -> impl IntoResponse {
    if password != state.password {
        error!("Incorrect password");
        return (StatusCode::UNAUTHORIZED, "Nope");
    }

    info!("Received light control request: {:?}", action);
    let controller = state.controller;

    let result = match action {
        LightAction::AllOff => controller.turn_all_off().await,
        LightAction::AllOn => {
            controller
                .turn_all_on(Brightness::Medium, ColorTemp::Warm)
                .await
        }
        LightAction::NightMode => controller.night_mode().await,
        LightAction::MovieMode => controller.movie_mode().await,
        LightAction::PartyMode => controller.party_mode().await,
    };

    if let Err(e) = result {
        error!("ZigbeeController action failed: {}", e);
        return (StatusCode::OK, "Controller action failed");
    }

    (StatusCode::OK, "OK")
}

async fn get_css() -> impl IntoResponse {
    let css = tokio::fs::read_to_string(format!("{}/style.css", HTML_DIR))
        .await
        .unwrap_or_else(|_| String::from("body { font-family: sans-serif; }"));

    (StatusCode::OK, [("Content-Type", "text/css")], css)
}

async fn get_js() -> impl IntoResponse {
    let js = tokio::fs::read_to_string(format!("{}/script.js", HTML_DIR))
        .await
        .unwrap_or_else(|_| String::from("console.log('Error loading script');"));

    (
        StatusCode::OK,
        [("Content-Type", "application/javascript")],
        js,
    )
}
