use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use wezztershier_core::{
    lexer::Lexer,
    parser::{parse_annotations, Parser},
    widgets::{factory::WidgetBuilder, implementations::register_core_widgets},
};

const SMALL_CONFIG: &str = r#"
-- <<TUNER-START>>
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 14
-- @ui: select(options="Dark, Light") type=string
config.theme = "Dark"
-- <<TUNER-END>>
"#;

const MEDIUM_CONFIG: &str = r#"
-- <<TUNER-START>>
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 14

-- @ui: theme_selector(themes="builtin", filter="all") type=string
config.color_scheme = "dracula"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.background = "#282a36"

-- @ui: color_picker(format="rgba", alpha=true) type=color  
config.colors.foreground = "rgba(248, 248, 242, 1.0)"

-- @ui: select(options="bottom, top, hidden") type=string
config.tab_bar_at_bottom = "bottom"

-- @ui: slider(min=0.1, max=2.0, step=0.1) type=float
config.window_background_opacity = 0.95

-- @ui: color_picker(format="hsl", alpha=false) type=color
config.colors.cursor_bg = "hsl(250, 100%, 80%)"

-- @ui: select(options="Block, Underline, Bar") type=string
config.default_cursor_style = "Block"
-- <<TUNER-END>>
"#;

const LARGE_CONFIG: &str = r#"
-- <<TUNER-START>>
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 14

-- @ui: theme_selector(themes="builtin", filter="all") type=string
config.color_scheme = "dracula"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.background = "#282a36"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.foreground = "#f8f8f2"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.cursor_bg = "#f8f8f2"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.cursor_border = "#f8f8f2"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.selection_fg = "#f8f8f2"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.selection_bg = "#44475a"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.ansi[0] = "#21222c"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.ansi[1] = "#ff5555"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.ansi[2] = "#50fa7b"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.ansi[3] = "#f1fa8c"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.ansi[4] = "#bd93f9"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.ansi[5] = "#ff79c6"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.ansi[6] = "#8be9fd"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.ansi[7] = "#f8f8f2"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.brights[0] = "#6272a4"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.brights[1] = "#ff6e6e"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.brights[2] = "#69ff94"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.brights[3] = "#ffffa5"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.brights[4] = "#d6acff"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.brights[5] = "#ff92df"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.brights[6] = "#a4ffff"
-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.brights[7] = "#ffffff"

-- @ui: select(options="bottom, top, hidden") type=string
config.tab_bar_at_bottom = "bottom"
-- @ui: select(options="Block, Underline, Bar") type=string
config.default_cursor_style = "Block"
-- @ui: slider(min=0.1, max=2.0, step=0.1) type=float
config.window_background_opacity = 0.95
-- @ui: slider(min=0, max=100, step=5) type=int
config.max_fps = 60
-- @ui: slider(min=1000, max=10000, step=100) type=int
config.scrollback_lines = 3500
-- <<TUNER-END>>
"#;

fn lexer_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    
    for (name, config) in [
        ("small", SMALL_CONFIG),
        ("medium", MEDIUM_CONFIG),
        ("large", LARGE_CONFIG),
    ] {
        group.throughput(Throughput::Bytes(config.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokenize", name), config, |b, config| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(config));
                let mut token_count = 0;
                while let Some(token) = lexer.next_token() {
                    black_box(&token);
                    token_count += 1;
                    if token.token_type == wezzte_core::lexer::TokenType::Eof {
                        break;
                    }
                }
                token_count
            })
        });
    }
    group.finish();
}

fn parser_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    
    for (name, config) in [
        ("small", SMALL_CONFIG),
        ("medium", MEDIUM_CONFIG),
        ("large", LARGE_CONFIG),
    ] {
        group.throughput(Throughput::Bytes(config.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), config, |b, config| {
            b.iter(|| {
                parse_annotations(black_box(config)).unwrap()
            })
        });
    }
    group.finish();
}

fn widget_creation_benchmark(c: &mut Criterion) {
    // Setup: register widgets once
    register_core_widgets().expect("Failed to register core widgets");
    
    let mut group = c.benchmark_group("widget_creation");
    
    for (name, config) in [
        ("small", SMALL_CONFIG),
        ("medium", MEDIUM_CONFIG),
        ("large", LARGE_CONFIG),
    ] {
        let entries = parse_annotations(config).unwrap();
        
        group.throughput(Throughput::Elements(entries.len() as u64));
        group.bench_with_input(BenchmarkId::new("create_widgets", name), &entries, |b, entries| {
            b.iter(|| {
                let mut builder = WidgetBuilder::new();
                builder.add_from_entries(black_box(entries)).unwrap();
                builder
            })
        });
    }
    group.finish();
}

fn end_to_end_benchmark(c: &mut Criterion) {
    // Setup: register widgets once
    register_core_widgets().expect("Failed to register core widgets");
    
    let mut group = c.benchmark_group("end_to_end");
    
    for (name, config) in [
        ("small", SMALL_CONFIG),
        ("medium", MEDIUM_CONFIG),
        ("large", LARGE_CONFIG),
    ] {
        group.throughput(Throughput::Bytes(config.len() as u64));
        group.bench_with_input(BenchmarkId::new("full_pipeline", name), config, |b, config| {
            b.iter(|| {
                // Parse annotations
                let entries = parse_annotations(black_box(config)).unwrap();
                
                // Create widgets
                let mut builder = WidgetBuilder::new();
                builder.add_from_entries(&entries).unwrap();
                
                // Generate layout
                builder.generate_auto_layout();
                
                // Serialize for frontend
                let _serialized = builder.serialize_app_data().unwrap();
                
                builder
            })
        });
    }
    group.finish();
}

fn memory_usage_benchmark(c: &mut Criterion) {
    register_core_widgets().expect("Failed to register core widgets");
    
    let mut group = c.benchmark_group("memory_usage");
    
    // Test memory allocation patterns
    group.bench_function("repeated_parsing", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let entries = parse_annotations(black_box(MEDIUM_CONFIG)).unwrap();
                black_box(entries);
            }
        })
    });
    
    group.bench_function("widget_builder_reuse", |b| {
        b.iter(|| {
            let mut builder = WidgetBuilder::new();
            for _ in 0..10 {
                let entries = parse_annotations(black_box(SMALL_CONFIG)).unwrap();
                builder.add_from_entries(&entries).unwrap();
                builder.generate_config(); // Force serialization
            }
            builder
        })
    });
    
    group.finish();
}

fn color_parsing_benchmark(c: &mut Criterion) {
    use wezztershier_core::widgets::implementations::color_picker::ColorPickerWidget;
    
    let mut group = c.benchmark_group("color_parsing");
    
    let test_colors = [
        ("#ff0000", "hex"),
        ("rgb(255, 0, 0)", "rgb"),
        ("rgba(255, 0, 0, 0.5)", "rgba"),
        ("hsl(0, 100%, 50%)", "hsl"),
        ("hsla(0, 100%, 50%, 0.8)", "hsla"),
    ];
    
    for (color, format) in test_colors {
        group.bench_with_input(
            BenchmarkId::new("parse_color", format),
            color,
            |b, color| {
                b.iter(|| match format {
                    "hex" => ColorPickerWidget::parse_hex_color(black_box(color)),
                    "rgb" => ColorPickerWidget::parse_rgb_color(black_box(color)),
                    "rgba" => ColorPickerWidget::parse_rgba_color(black_box(color)),
                    "hsl" => ColorPickerWidget::parse_hsl_color(black_box(color)),
                    _ => None,
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    lexer_benchmark,
    parser_benchmark,
    widget_creation_benchmark,
    end_to_end_benchmark,
    memory_usage_benchmark,
    color_parsing_benchmark
);

criterion_main!(benches);