use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::{cors::CorsLayer, services::ServeDir};
use wezztershier_core::{
    parser::parse_annotations,
    widgets::{factory::WidgetBuilder, implementations::register_core_widgets, traits::WidgetValue},
};

#[derive(Debug, Serialize, Deserialize)]
struct ConfigRequest {
    config_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WidgetUpdateRequest {
    widget_id: String,
    value: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    widgets: serde_json::Value,
    config_preview: String,
}

static mut WIDGET_BUILDER: Option<WidgetBuilder> = None;

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn parse_config(Json(payload): Json<ConfigRequest>) -> Result<Json<ApiResponse>, StatusCode> {
    // Parse the configuration file
    let entries = parse_annotations(&payload.config_content)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Register core widgets
    register_core_widgets().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create widgets from parsed entries
    let mut builder = WidgetBuilder::new();
    builder.add_from_entries(&entries)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate automatic layout
    builder.generate_auto_layout();

    // Serialize complete application data including layout
    let app_data = builder.serialize_app_data()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let widgets = app_data.get("widgets").cloned().unwrap_or_default();
    let config_preview = app_data.get("config_preview")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Store builder globally (unsafe but simple for demo)
    unsafe {
        WIDGET_BUILDER = Some(builder);
    }

    Ok(Json(ApiResponse {
        widgets,
        config_preview,
    }))
}

async fn update_widget(Json(payload): Json<WidgetUpdateRequest>) -> Result<Json<ApiResponse>, StatusCode> {
    unsafe {
        if let Some(builder) = WIDGET_BUILDER.as_mut() {
            // Convert JSON value to WidgetValue
            let widget_value = match payload.value {
                serde_json::Value::String(s) => WidgetValue::String(s),
                serde_json::Value::Number(n) => WidgetValue::Number(n.as_f64().unwrap_or(0.0)),
                serde_json::Value::Bool(b) => WidgetValue::Boolean(b),
                _ => return Err(StatusCode::BAD_REQUEST),
            };

            // Update the widget
            builder.update_widget_value(&payload.widget_id, widget_value)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Return updated application data
            let app_data = builder.serialize_app_data()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let widgets = app_data.get("widgets").cloned().unwrap_or_default();
            let config_preview = app_data.get("config_preview")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            Ok(Json(ApiResponse {
                widgets,
                config_preview,
            }))
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn get_sample_config() -> Json<String> {
    Json(String::from(
        "-- Sample WezTerm configuration with GUI annotations\n\
         -- @ui: slider(min=8, max=72, step=1) type=int\n\
         config.font_size = 14\n\
         \n\
         -- @ui: theme_selector(themes=builtin, filter=all) type=string\n\
         config.color_scheme = \"dracula\"\n\
         \n\
         -- @ui: color_picker(format=hex, alpha=false) type=color\n\
         config.colors.background = \"#282a36\"\n\
         \n\
         -- @ui: slider(min=0.1, max=2.0, step=0.1) type=float\n\
         config.window_background_opacity = 0.95"
    ))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/parse", post(parse_config))
        .route("/api/update", post(update_widget))
        .route("/api/sample", get(get_sample_config))
        .nest_service("/static", ServeDir::new("src-web/static"))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to address");

    println!("🎨 Wezztershier Web GUI running at http://localhost:8080");
    println!("   Open this URL in your browser to configure WezTerm visually!");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}