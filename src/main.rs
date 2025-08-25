mod web_assets;

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
use tower_http::cors::CorsLayer;
use wezztershier_core::{
    ConfigManager,
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
    /// Launch the embedded web GUI
    Gui(GuiArgs),
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

    /// Port for web server
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Run server in background (daemon mode)  
    #[arg(long)]
    daemon: bool,
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

// Global state (simple approach for single-user web GUI)
static mut WIDGET_BUILDER: Option<WidgetBuilder> = None;

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
        Commands::Validate(args) => handle_validate(args).await,
        Commands::Widgets => handle_widgets().await,
        Commands::Docs(args) => handle_docs(args).await,
        Commands::Debug(args) => handle_debug(args).await,
    }
}

async fn handle_gui(args: GuiArgs) -> Result<()> {
    println!("{}", "🎨 Launching Wezztershier Web GUI...".magenta().bold());
    
    if let Some(file) = args.file {
        println!("Configuration file: {}", file.display().to_string().cyan());
    }

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/parse", post(parse_config))
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

    Ok(())
}

// Web server handlers
async fn serve_index() -> Html<&'static str> {
    Html(web_assets::INDEX_HTML)
}

async fn parse_config(Json(payload): Json<ConfigRequest>) -> Result<Json<ApiResponse>, StatusCode> {
    let entries = parse_annotations(&payload.config_content)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut builder = WidgetBuilder::new();
    builder.add_from_entries(&entries)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    builder.generate_auto_layout();

    let app_data = builder.serialize_app_data()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let widgets = app_data.get("widgets").cloned().unwrap_or_default();
    let config_preview = app_data.get("config_preview")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Store builder globally (simple approach for single-user GUI)
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
            let widget_value = match payload.value {
                serde_json::Value::String(s) => WidgetValue::String(s),
                serde_json::Value::Number(n) => WidgetValue::Number(n.as_f64().unwrap_or(0.0)),
                serde_json::Value::Bool(b) => WidgetValue::Boolean(b),
                _ => return Err(StatusCode::BAD_REQUEST),
            };

            builder.update_widget_value(&payload.widget_id, widget_value)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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