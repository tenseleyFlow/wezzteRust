use tauri::{Manager, State};
use wezztershier_core::{
    parser::parse_annotations,
    widgets::{factory::WidgetBuilder, implementations::register_core_widgets, traits::WidgetValue},
};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

// Global application state
type AppState = Arc<Mutex<Option<WidgetBuilder>>>;

#[derive(Debug, Serialize, Deserialize)]
struct WidgetData {
    widgets: serde_json::Value,
    config_preview: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParseConfigRequest {
    config_content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateWidgetRequest {
    widget_id: String,
    value: serde_json::Value,
}

#[tauri::command]
async fn parse_config(
    request: ParseConfigRequest,
    state: State<'_, AppState>,
) -> Result<WidgetData, String> {
    // Parse the configuration file
    let entries = parse_annotations(&request.config_content)
        .map_err(|e| format!("Parse error: {}", e))?;

    // Register core widgets
    register_core_widgets().map_err(|e| format!("Widget registration error: {}", e))?;

    // Create widgets from parsed entries
    let mut builder = WidgetBuilder::new();
    builder.add_from_entries(&entries)
        .map_err(|e| format!("Widget creation error: {}", e))?;

    // Generate automatic layout
    builder.generate_auto_layout();

    // Serialize complete application data including layout
    let app_data = builder.serialize_app_data()
        .map_err(|e| format!("Serialization error: {}", e))?;

    let widgets = app_data.get("widgets").cloned().unwrap_or_default();
    let config_preview = app_data.get("config_preview")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Store builder in state for updates
    let mut app_state = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    *app_state = Some(builder);

    Ok(WidgetData {
        widgets,
        config_preview,
    })
}

#[tauri::command]
async fn update_widget(
    request: UpdateWidgetRequest,
    state: State<'_, AppState>,
) -> Result<WidgetData, String> {
    let mut app_state = state.lock().map_err(|e| format!("State lock error: {}", e))?;
    
    if let Some(builder) = app_state.as_mut() {
        // Convert JSON value to WidgetValue
        let widget_value = match request.value {
            serde_json::Value::String(s) => WidgetValue::String(s),
            serde_json::Value::Number(n) => WidgetValue::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::Bool(b) => WidgetValue::Boolean(b),
            _ => return Err("Unsupported value type".to_string()),
        };

        // Update the widget
        builder.update_widget_value(&request.widget_id, widget_value)
            .map_err(|e| format!("Widget update error: {}", e))?;

        // Return updated application data
        let app_data = builder.serialize_app_data()
            .map_err(|e| format!("Serialization error: {}", e))?;

        let widgets = app_data.get("widgets").cloned().unwrap_or_default();
        let config_preview = app_data.get("config_preview")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(WidgetData {
            widgets,
            config_preview,
        })
    } else {
        Err("No configuration loaded".to_string())
    }
}

#[tauri::command]
async fn get_sample_config() -> Result<String, String> {
    Ok(String::from(
        "-- Sample WezTerm configuration with GUI annotations\n\
         -- @ui: slider(min=8, max=72, step=1) type=int\n\
         config.font_size = 14\n\
         \n\
         -- @ui: theme_selector(themes=builtin, filter=all) type=string\n\
         config.color_scheme = \"dracula\"\n\
         \n\
         -- @ui: color_picker(format=hex, alpha=false) type=color\n\
         config.colors.background = \"#282a36\""
    ))
}

fn main() {
    tracing_subscriber::fmt::init();

    let app_state: AppState = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            parse_config,
            update_widget,
            get_sample_config
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}