use crate::config::Config;
use crate::engine::{AgentEngine, ChatSessionRequest};
use axum::{
    extract::{State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

pub fn create_router(engine: Arc<AgentEngine>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/config", get(get_config_handler).post(update_config_handler))
        .route("/api/models", get(list_models_handler))
        .route("/api/tools", get(list_tools_handler))
        .route("/api/chat", post(chat_handler))
        .route("/api/ws", get(ws_handler))
        .nest_service("/", ServeDir::new("frontend/dist"))
        .layer(cors)
        .with_state(engine)
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "hermes-claw",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn get_config_handler(State(engine): State<Arc<AgentEngine>>) -> impl IntoResponse {
    let cfg = engine.get_config().await;
    Json(cfg)
}

async fn update_config_handler(
    State(engine): State<Arc<AgentEngine>>,
    Json(new_cfg): Json<Config>,
) -> impl IntoResponse {
    engine.update_config(new_cfg.clone()).await;
    Json(json!({ "status": "updated", "config": new_cfg }))
}

async fn list_models_handler(State(engine): State<Arc<AgentEngine>>) -> impl IntoResponse {
    let models = engine.list_all_models().await;
    Json(models)
}

async fn list_tools_handler(State(engine): State<Arc<AgentEngine>>) -> impl IntoResponse {
    let tools = engine.get_tool_registry().get_definitions();
    Json(tools)
}

async fn chat_handler(
    State(engine): State<Arc<AgentEngine>>,
    Json(payload): Json<ChatSessionRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match engine.process_chat(payload).await {
        Ok(res) => Ok(Json(res)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err)),
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(engine): State<Arc<AgentEngine>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, engine))
}

async fn handle_websocket(mut socket: WebSocket, engine: Arc<AgentEngine>) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            if let Ok(req) = serde_json::from_str::<ChatSessionRequest>(&text) {
                match engine.process_chat(req).await {
                    Ok(res) => {
                        let _ = socket
                            .send(Message::Text(
                                serde_json::to_string(&res).unwrap_or_default(),
                            ))
                            .await;
                    }
                    Err(err) => {
                        let err_json = json!({ "error": err });
                        let _ = socket
                            .send(Message::Text(err_json.to_string()))
                            .await;
                    }
                }
            }
        }
    }
}
