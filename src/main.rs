mod web_assets;
mod gui;
mod config;

use anyhow::{Context, Result};
use axum::{
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use clap::{Args, Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use wezztershier_core::{
    ConfigManager,
    config::{BackupManager, ConfigUpdater},
    parser::parse_annotations,
    widgets::{factory::WidgetBuilder, implementations::register_core_widgets, traits::WidgetValue},
};

#[derive(Parser)]
#[command(name = "wezztershier")]
#[command(about = "A beautiful GUI generator for WezTerm configuration files")]
#[command(version)]
struct Cli {
    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a configuration file and display widgets
    Parse(ParseArgs),
    /// Launch the GUI (respects user preference or use --native/--web)
    Gui(GuiArgs),
    /// Configure wezztershier settings
    Configure(ConfigureArgs),
    /// Validate a configuration file
    Validate(ValidateArgs),
    /// List available widget types
    Widgets,
    /// Generate documentation for widgets
    Docs(DocsArgs),
    /// Debug configuration parsing
    Debug(DebugArgs),
}

#[derive(Args)]
struct ParseArgs {
    /// Configuration file to parse
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Output format (json, table, yaml)
    #[arg(short, long, default_value = "table")]
    format: String,

    /// Generate automatic layout
    #[arg(short, long)]
    layout: bool,
}

#[derive(Args)]
struct GuiArgs {
    /// Configuration file to load
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Use native GUI instead of web interface (overrides default preference)
    #[arg(long)]
    native: bool,
    
    /// Use web GUI instead of native interface (overrides default preference)
    #[arg(long, conflicts_with = "native")]
    web: bool,

    /// Port for web server (web mode only)
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Run server in background (daemon mode, web mode only)  
    #[arg(long)]
    daemon: bool,
}

#[derive(Args)]
struct ConfigureArgs {
    /// Set default GUI backend (native or web)
    #[arg(long, value_name = "BACKEND")]
    set_default_gui: Option<String>,

    /// Show current configuration
    #[arg(long)]
    show: bool,

    /// Reset all settings to defaults
    #[arg(long)]
    reset: bool,
}

#[derive(Args)]
struct ValidateArgs {
    /// Configuration file to validate
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Show all validation details
    #[arg(short, long)]
    all: bool,
}

#[derive(Args)]
struct DocsArgs {
    /// Widget type to document (optional)
    #[arg(value_name = "WIDGET_TYPE")]
    widget_type: Option<String>,

    /// Output format (markdown, json, text)
    #[arg(short, long, default_value = "markdown")]
    format: String,
}

#[derive(Args)]
struct DebugArgs {
    /// Configuration file to debug
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Show lexer tokens
    #[arg(long)]
    tokens: bool,

    /// Show AST structure
    #[arg(long)]
    ast: bool,

    /// Show widget creation process
    #[arg(long)]
    widgets: bool,
}

// Web server types
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

// Global state for web GUI with file writeback support
struct AppState {
    widget_builder: WidgetBuilder,
    config_manager: Option<ConfigManager>,
    backup_manager: Option<BackupManager>,
    config_content: String,
    last_update: Instant,
}

impl AppState {
    fn new() -> Self {
        Self {
            widget_builder: WidgetBuilder::new(),
            config_manager: None,
            backup_manager: None,
            config_content: String::new(),
            last_update: Instant::now(),
        }
    }
}

static APP_STATE: Mutex<Option<AppState>> = Mutex::const_new(None);

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let level = if cli.debug {
        "debug"
    } else if cli.verbose {
        "info"
    } else {
        "warn"
    };

    tracing_subscriber::fmt()
        .with_env_filter(format!("wezztershier={},wezztershier_core={}", level, level))
        .init();

    // Register core widgets
    register_core_widgets().context("Failed to register core widgets")?;

    match cli.command {
        Commands::Parse(args) => handle_parse(args).await,
        Commands::Gui(args) => handle_gui(args).await,
        Commands::Configure(args) => handle_configure(args).await,
        Commands::Validate(args) => handle_validate(args).await,
        Commands::Widgets => handle_widgets().await,
        Commands::Docs(args) => handle_docs(args).await,
        Commands::Debug(args) => handle_debug(args).await,
    }
}

async fn handle_gui(args: GuiArgs) -> Result<()> {
    // Determine which GUI to use based on args and user preferences
    let use_native = if args.native {
        true
    } else if args.web {
        false
    } else {
        // Check user preference
        match config::ConfigManager::load() {
            Ok(config_manager) => config_manager.config().default_gui_backend == config::GuiBackend::Native,
            Err(_) => true, // Default to native if config fails to load
        }
    };

    if use_native {
        println!("{}", "🎨 Launching Wezztershier Native GUI...".magenta().bold());
        if !args.native {
            println!("{}", "   (Using user preference. Use --web to override)".dimmed());
        }
        
        if let Some(file) = &args.file {
            println!("Configuration file: {}", file.display().to_string().cyan());
        }
        
        // Launch native GUI (blocking call)
        gui::run_native_gui(args.file.clone())
            .context("Failed to start native GUI")?;
    } else {
        println!("{}", "🎨 Launching Wezztershier Web GUI...".magenta().bold());
        if !args.web {
            println!("{}", "   (Using user preference. Use --native to override)".dimmed());
        }
        
        if let Some(file) = args.file {
            println!("Configuration file: {}", file.display().to_string().cyan());
        }

        let app = Router::new()
            .route("/", get(serve_index))
            .route("/api/parse", post(parse_config))
            .route("/api/load", post(load_config_file))
            .route("/api/update", post(update_widget))
            .route("/api/sample", get(get_sample_config))
            .layer(CorsLayer::permissive());

        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", args.port))
            .await
            .context("Failed to bind to address")?;

        println!("{}Server running at: {}", "🌐 ".green(), format!("http://localhost:{}", args.port).blue().underline());
        
        if !args.daemon {
            println!("{}Press Ctrl+C to stop the server", "💡 ".yellow());
        }

        axum::serve(listener, app)
            .await
            .context("Failed to start web server")?;
    }

    Ok(())
}

async fn handle_configure(args: ConfigureArgs) -> Result<()> {
    if args.show {
        // Show current configuration
        match config::ConfigManager::load() {
            Ok(config_manager) => {
                println!("{}", "🔧 Wezztershier Configuration".cyan().bold());
                println!("Configuration file: {}", config_manager.path().display());
                println!();
                println!("Current Settings:");
                println!("  Default GUI Backend: {}", 
                    config_manager.config().default_gui_backend.as_str().yellow());
            }
            Err(e) => {
                println!("{}", "⚠️  No configuration found, using defaults".yellow());
                println!("  Default GUI Backend: {}", config::GuiBackend::Native.as_str().yellow());
                println!("  Error: {}", e);
            }
        }
        return Ok(());
    }

    if args.reset {
        // Reset configuration to defaults
        println!("{}", "🔄 Resetting configuration to defaults...".blue());
        let mut config_manager = config::ConfigManager::load()
            .unwrap_or_else(|_| {
                let path = dirs::config_dir()
                    .unwrap_or_else(|| std::env::temp_dir())
                    .join("wezztershier")
                    .join("config.toml");
                config::ConfigManager::new(path, config::WezztershierConfig::default())
            });
        
        config_manager.reset()
            .context("Failed to reset configuration")?;
        
        println!("{}", "✅ Configuration reset to defaults".green());
        println!("  Default GUI Backend: {}", config::GuiBackend::Native.as_str().yellow());
        return Ok(());
    }

    if let Some(backend_str) = args.set_default_gui {
        // Set default GUI backend
        let backend = config::GuiBackend::from_str(&backend_str)
            .context("Invalid GUI backend")?;

        let mut config_manager = config::ConfigManager::load()
            .unwrap_or_else(|_| {
                println!("{}", "Creating new configuration file...".blue());
                // Create with defaults
                let path = dirs::config_dir()
                    .unwrap_or_else(|| std::env::temp_dir())
                    .join("wezztershier")
                    .join("config.toml");
                config::ConfigManager::new(path, config::WezztershierConfig::default())
            });

        config_manager.config_mut().default_gui_backend = backend;
        config_manager.save()
            .context("Failed to save configuration")?;

        println!("{}", "✅ Configuration updated".green());
        println!("  Default GUI Backend: {}", backend.as_str().yellow());
        println!("  Saved to: {}", config_manager.path().display());
        return Ok(());
    }

    // No specific action, show help
    println!("{}", "🔧 Wezztershier Configure".cyan().bold());
    println!("Use one of the following options:");
    println!("  --show                     Show current configuration");
    println!("  --set-default-gui native   Set native GUI as default");
    println!("  --set-default-gui web      Set web GUI as default");
    println!("  --reset                    Reset all settings to defaults");
    
    Ok(())
}

// Web server handlers
async fn serve_index() -> Html<&'static str> {
    Html(web_assets::INDEX_HTML)
}

async fn parse_config(Json(payload): Json<ConfigRequest>) -> Result<Json<ApiResponse>, StatusCode> {
    let entries = parse_annotations(&payload.config_content)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut state_guard = APP_STATE.lock().await;
    let mut state = state_guard.take().unwrap_or_else(AppState::new);
    
    state.widget_builder = WidgetBuilder::new();
    state.widget_builder.add_from_entries(&entries)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.widget_builder.generate_auto_layout();
    state.config_content = payload.config_content;

    let app_data = state.widget_builder.serialize_app_data()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let widgets = app_data.get("widgets").cloned().unwrap_or_default();
    let config_preview = app_data.get("config_preview")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    *state_guard = Some(state);

    Ok(Json(ApiResponse {
        widgets,
        config_preview,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
struct LoadConfigRequest {
    file_path: String,
}

async fn load_config_file(Json(payload): Json<LoadConfigRequest>) -> Result<Json<ApiResponse>, StatusCode> {
    let config_path = PathBuf::from(&payload.file_path);
    
    let config_manager = ConfigManager::new(&config_path);
    let config_content = config_manager.read_config().await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let backup_manager = BackupManager::new(&config_path, None, 10)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Create initial backup
    let _backup_path = backup_manager.create_backup("gui-session").await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let entries = parse_annotations(&config_content)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut state_guard = APP_STATE.lock().await;
    let mut state = state_guard.take().unwrap_or_else(AppState::new);
    
    state.config_manager = Some(config_manager);
    state.backup_manager = Some(backup_manager);
    state.widget_builder = WidgetBuilder::new();
    state.widget_builder.add_from_entries(&entries)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    state.widget_builder.generate_auto_layout();
    state.config_content = config_content;

    let app_data = state.widget_builder.serialize_app_data()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let widgets = app_data.get("widgets").cloned().unwrap_or_default();
    let config_preview = app_data.get("config_preview")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    *state_guard = Some(state);

    Ok(Json(ApiResponse {
        widgets,
        config_preview,
    }))
}

async fn update_widget(Json(payload): Json<WidgetUpdateRequest>) -> Result<Json<ApiResponse>, StatusCode> {
    // Prepare writeback data outside the lock
    let (config_manager_opt, content, last_update, response) = {
        let mut state_guard = APP_STATE.lock().await;
        let state = state_guard.as_mut().ok_or(StatusCode::BAD_REQUEST)?;
        
        let widget_value = match payload.value {
            serde_json::Value::String(s) => WidgetValue::String(s),
            serde_json::Value::Number(n) => WidgetValue::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::Bool(b) => WidgetValue::Boolean(b),
            _ => return Err(StatusCode::BAD_REQUEST),
        };

        state.widget_builder.update_widget_value(&payload.widget_id, widget_value)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Update timestamp for debouncing
        state.last_update = Instant::now();

        let app_data = state.widget_builder.serialize_app_data()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let widgets = app_data.get("widgets").cloned().unwrap_or_default();
        let config_preview = app_data.get("config_preview")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let response = ApiResponse {
            widgets,
            config_preview,
        };

        // Clone necessary data for writeback
        (
            state.config_manager.clone(),
            state.config_content.clone(), 
            state.last_update,
            response
        )
    };

    // Handle file writeback outside the lock
    if let Some(config_manager) = config_manager_opt {
        let generated_config = {
            let state_guard = APP_STATE.lock().await;
            let state = state_guard.as_ref().ok_or(StatusCode::BAD_REQUEST)?;
            state.widget_builder.generate_config()
        };
        
        let config_updater = ConfigUpdater::new(content);
        
        match config_updater.update_tuner_block(&generated_config) {
            Ok(updated_content) => {
                // Update stored content first
                {
                    let mut state_guard = APP_STATE.lock().await;
                    if let Some(state) = state_guard.as_mut() {
                        state.config_content = updated_content.clone();
                    }
                }
                
                // Spawn writeback task with debouncing
                tokio::spawn(async move {
                    // Wait for debounce period
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    // Check if this is still the latest update
                    let current_state_guard = APP_STATE.lock().await;
                    if let Some(current_state) = current_state_guard.as_ref() {
                        if current_state.last_update == last_update {
                            // Write to file - this will trigger WezTerm reload
                            if let Err(e) = config_manager.write_config(&updated_content).await {
                                tracing::error!("Failed to write config: {}", e);
                            }
                        }
                    }
                });
            }
            Err(e) => {
                tracing::error!("Failed to update tuner block: {}", e);
            }
        }
    }

    Ok(Json(response))
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

// CLI command handlers (from original CLI code)
async fn handle_parse(args: ParseArgs) -> Result<()> {
    println!("{}", "📝 Parsing configuration file...".cyan().bold());

    let _config_manager = ConfigManager::new(&args.file);
    let content = tokio::fs::read_to_string(&args.file)
        .await
        .context("Failed to read configuration file")?;

    let entries = parse_annotations(&content)
        .context("Failed to parse configuration annotations")?;

    println!("{}", format!("✅ Found {} widget definitions", entries.len()).green());

    match args.format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&entries)?;
            println!("{}", json);
        }
        "yaml" => {
            println!("YAML output not yet implemented");
        }
        "table" | _ => {
            print_widgets_table(&entries, args.layout).await?;
        }
    }

    Ok(())
}

async fn print_widgets_table(entries: &[wezztershier_core::ConfigEntry], show_layout: bool) -> Result<()> {
    if entries.is_empty() {
        println!("{}", "No widgets found in configuration".yellow());
        return Ok(());
    }

    println!("\n{}", "Widget Summary:".bold().underline());
    println!("{:<20} {:<15} {:<30} {:<20}", "Widget ID".bold(), "Type".bold(), "Config Key".bold(), "Current Value".bold());
    println!("{}", "─".repeat(85));

    let mut builder = WidgetBuilder::new();
    builder.add_from_entries(entries)?;

    for widget_id in builder.widget_ids() {
        if let Some(widget) = builder.get_widget(&widget_id) {
            let widget_type = detect_widget_type(widget);
            println!(
                "{:<20} {:<15} {:<30} {:<20}",
                widget_id.blue(),
                widget_type.yellow(),
                widget.config().config_key.green(),
                format!("{:?}", widget.get_value()).cyan()
            );
        }
    }

    if show_layout {
        println!("\n{}", "Generated Layout:".bold().underline());
        for widget_id in builder.widget_ids() {
            if let Some(widget) = builder.get_widget(&widget_id) {
                println!("  📄 Widget: {}", widget_id.cyan());
                println!("    Config: {}", widget.config().config_key.green());
            }
        }
    }

    Ok(())
}

fn detect_widget_type(widget: &dyn wezztershier_core::widgets::traits::Widget) -> &str {
    let render_data = widget.render_data();
    if render_data.contains_key("slider") {
        "slider"
    } else if render_data.contains_key("select") {
        "select"
    } else if render_data.contains_key("color_picker") {
        "color_picker"
    } else if render_data.contains_key("theme_selector") {
        "theme_selector"
    } else {
        "unknown"
    }
}

async fn handle_validate(args: ValidateArgs) -> Result<()> {
    println!("{}", "🔍 Validating configuration...".blue().bold());

    let _config_manager = ConfigManager::new(&args.file);
    let content = tokio::fs::read_to_string(&args.file)
        .await
        .context("Failed to read configuration file")?;

    match parse_annotations(&content) {
        Ok(entries) => {
            println!("{}", "✅ Configuration is valid!".green().bold());
            
            if args.all {
                println!("\nValidation Details:");
                println!("- {} widget definitions found", entries.len());
                println!("- All annotations parsed successfully");
                println!("- All widget types are supported");
                
                let mut builder = WidgetBuilder::new();
                match builder.add_from_entries(&entries) {
                    Ok(_) => println!("- All widgets created successfully"),
                    Err(e) => println!("- {}: {}", "Widget creation warning".yellow(), e),
                }
            }
        }
        Err(e) => {
            println!("{}", "❌ Configuration validation failed!".red().bold());
            println!("Error: {}", e.to_string().red());
        }
    }

    Ok(())
}

async fn handle_widgets() -> Result<()> {
    println!("{}", "🛠️  Available Widget Types:".cyan().bold());
    
    let metadata = wezztershier_core::widgets::metadata::CoreWidgets::all();
    
    for (widget_type, meta) in metadata {
        println!("\n{}", format!("📋 {}", meta.name).yellow().bold());
        println!("   Type: {}", widget_type.blue());
        println!("   Category: {}", meta.category.green());
        println!("   Description: {}", meta.description.dimmed());
        
        if !meta.examples.is_empty() {
            println!("   Example: {}", meta.examples[0].decorator.cyan());
        }
    }

    Ok(())
}

async fn handle_docs(args: DocsArgs) -> Result<()> {
    let metadata = wezztershier_core::widgets::metadata::CoreWidgets::all();
    
    if let Some(widget_type) = args.widget_type {
        if let Some(meta) = metadata.get(&widget_type) {
            match args.format.as_str() {
                "json" => {
                    println!("{}", serde_json::to_string_pretty(meta)?);
                }
                "text" => {
                    println!("{}", meta.generate_docs());
                }
                "markdown" | _ => {
                    println!("{}", meta.generate_docs());
                }
            }
        } else {
            println!("{}", format!("Widget type '{}' not found", widget_type).red());
        }
    } else {
        println!("{}", "📚 Widget Documentation".cyan().bold());
        
        for (_, meta) in metadata {
            println!("{}", meta.generate_docs());
            println!("---\n");
        }
    }

    Ok(())
}

async fn handle_debug(args: DebugArgs) -> Result<()> {
    println!("{}", "🐛 Debug Mode".red().bold());
    println!("File: {}", args.file.display().to_string().cyan());

    let _config_manager = ConfigManager::new(&args.file);
    let content = tokio::fs::read_to_string(&args.file)
        .await
        .context("Failed to read configuration file")?;

    if args.tokens {
        println!("\n{}", "🔤 Lexer Tokens:".yellow().bold());
        println!("  {}", "Lexer token iteration not yet implemented in CLI".dimmed());
    }

    if args.ast {
        println!("\n{}", "🌳 AST Structure:".green().bold());
        match parse_annotations(&content) {
            Ok(entries) => {
                for entry in entries {
                    println!("📄 Config Entry:");
                    println!("  Key: {}", entry.key.cyan());
                    println!("  Value: {}", entry.value.blue());
                    println!("  Annotation: {:#?}", entry.annotation);
                    println!();
                }
            }
            Err(e) => println!("Parse Error: {}", e.to_string().red()),
        }
    }

    if args.widgets {
        println!("\n{}", "🔧 Widget Creation Process:".magenta().bold());
        match parse_annotations(&content) {
            Ok(entries) => {
                let mut builder = WidgetBuilder::new();
                for entry in &entries {
                    match builder.add_from_entry(entry) {
                        Ok(widget_id) => {
                            println!("✅ Created widget: {}", widget_id.green());
                        }
                        Err(e) => {
                            println!("❌ Failed to create widget for '{}': {}", entry.key.red(), e);
                        }
                    }
                }
                
                println!("\nFinal widget count: {}", builder.len());
            }
            Err(e) => println!("Parse Error: {}", e.to_string().red()),
        }
    }

    Ok(())
}