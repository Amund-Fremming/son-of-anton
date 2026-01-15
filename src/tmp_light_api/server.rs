use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use ngrok::prelude::*;
use serde::Deserialize;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::{Arc, Mutex};
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
    logs: Arc<Mutex<Vec<(DateTime<Utc>, String)>>>,
}

impl WebState {
    pub fn log(&self, message: String) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.push((Utc::now(), message));

            // Keep only the last 20 logs
            if logs.len() > 20 {
                let excess = logs.len() - 20;
                logs.drain(0..excess);
            }
        }
    }

    pub fn get_logs(&self) -> Vec<(DateTime<Utc>, String)> {
        self.logs
            .lock()
            .map(|logs| logs.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
pub enum LightAction {
    AllOff,
    AllOn,
    NightMode,
    MovieMode,
    PartyMode,
}

// Server

pub async fn start_server(controller: ZigbeeController) -> Result<(), AppError> {
    info!("Starting light controller server");

    let password = std::env::var("PASSWORD")?;
    let state = WebState {
        controller,
        password,
        logs: Arc::new(Mutex::new(Vec::new())),
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
        .route("/logs/{password}", get(logs_page))
        .route("/api/logs/{password}", get(get_logs_api))
        .route("/set-light/{password}/{action}", post(set_light))
        .route("/style.css", get(get_css))
        .route("/script.js", get(get_js))
        .with_state(state);

    // Check if ngrok domain is configured
    let ngrok_domain = std::env::var("NGROK_DOMAIN").ok();

    if let Some(mut domain) = ngrok_domain {
        // Strip https:// or http:// if present
        domain = domain.replace("https://", "").replace("http://", "");

        info!("Setting up ngrok tunnel with domain: {}", domain);

        // Verify NGROK_AUTHTOKEN is set
        let auth_token = std::env::var("NGROK_AUTHTOKEN")
            .map_err(|_| AppError::Internal("NGROK_AUTHTOKEN environment variable not set. Get your token from https://dashboard.ngrok.com/get-started/your-authtoken".to_string()))?;

        if auth_token.is_empty() || auth_token == "FILLIN" {
            return Err(AppError::Internal("NGROK_AUTHTOKEN is not configured. Please set it to your actual ngrok auth token from https://dashboard.ngrok.com/get-started/your-authtoken".to_string()));
        }

        // Start local server first
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to bind local server: {}", e)))?;

        info!("Local server bound to {}", addr);

        // Create ngrok session and tunnel
        let session = ngrok::Session::builder()
            .authtoken_from_env()
            .connect()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create ngrok session: {}. Make sure NGROK_AUTHTOKEN is set correctly.", e)))?;

        // Start tunnel that forwards to local address
        let _tunnel = session
            .http_endpoint()
            .domain(&domain)
            .listen_and_forward(format!("http://127.0.0.1:3000").parse().unwrap())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create ngrok tunnel: {}", e)))?;

        info!("Ngrok tunnel established at: https://{}", domain);

        // Run the local server
        axum::serve(listener, app)
            .await
            .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;
    } else {
        // Run without ngrok - local only
        info!("No ngrok domain specified, running locally only");
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        info!("Server running on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app)
            .await
            .map_err(|e| AppError::Internal(format!("Server error: {}", e)))?;
    }

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
        state.log("Login failed: Incorrect password".to_string());
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
        state.log("Control page access failed: Incorrect password".to_string());
        return (StatusCode::UNAUTHORIZED, "Nope").into_response();
    }

    let html = tokio::fs::read_to_string(format!("{}/control.html", HTML_DIR))
        .await
        .unwrap_or_else(|e| {
            error!("Failed to load control.html: {}", e);
            state.log(format!("Failed to load control.html: {}", e));
            "Nope".to_string()
        });

    Html(html).into_response()
}

async fn logs_page(
    State(state): State<WebState>,
    Path(password): Path<String>,
) -> impl IntoResponse {
    info!("Serving logs page");
    if password != state.password {
        error!("Incorrect password");
        state.log("Logs page access failed: Incorrect password".to_string());
        return (StatusCode::UNAUTHORIZED, "Nope").into_response();
    }

    let html = tokio::fs::read_to_string(format!("{}/logs.html", HTML_DIR))
        .await
        .unwrap_or_else(|e| {
            error!("Failed to load logs.html: {}", e);
            state.log(format!("Failed to load logs.html: {}", e));
            "Nope".to_string()
        });

    Html(html).into_response()
}

async fn get_logs_api(
    State(state): State<WebState>,
    Path(password): Path<String>,
) -> impl IntoResponse {
    if password != state.password {
        error!("Incorrect password");
        state.log("Logs API access failed: Incorrect password".to_string());
        return (
            StatusCode::UNAUTHORIZED,
            Json(Vec::<(String, String)>::new()),
        )
            .into_response();
    }

    let logs = state.get_logs();
    let log_entries: Vec<(String, String)> = logs
        .into_iter()
        .map(|(timestamp, message)| {
            (
                timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                message,
            )
        })
        .collect();

    Json(log_entries).into_response()
}

async fn set_light(
    State(state): State<WebState>,
    Path((password, action)): Path<(String, LightAction)>,
) -> impl IntoResponse {
    if password != state.password {
        error!("Incorrect password");
        state.log("Set light failed: Incorrect password".to_string());
        return (StatusCode::UNAUTHORIZED, "Nope");
    }

    info!("Received light control request: {:?}", action);
    let controller = state.controller.clone();

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
        state.log(format!("ZigbeeController action failed: {}", e));
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
