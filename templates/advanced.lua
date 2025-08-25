-- Advanced WezTerm configuration with full Wezzte widget showcase
-- Copy this to ~/.config/wezterm/wezterm.lua and customize

local wezterm = require('wezterm')
local config = {}

-- <<TUNER-START>>

-- Font Configuration
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 14

-- Theme and Colors
-- @ui: theme_selector(themes="builtin", filter="all") type=string
config.color_scheme = "Tokyo Night"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.background = "#1a1b26"

-- @ui: color_picker(format="rgba", alpha=true) type=color
config.colors.foreground = "rgba(192, 202, 245, 1.0)"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.cursor_bg = "#c0caf5"

-- @ui: color_picker(format="hsl", alpha=false) type=color
config.colors.selection_bg = "hsl(230, 30%, 30%)"

-- Window Behavior
-- @ui: slider(min=0.1, max=1.0, step=0.05) type=float
config.window_background_opacity = 0.95

-- @ui: slider(min=500, max=50000, step=500) type=int
config.scrollback_lines = 10000

-- @ui: select(options="bottom, top, hidden") type=string
config.tab_bar_at_bottom = "bottom"

-- @ui: select(options="Block, Underline, Bar") type=string
config.default_cursor_style = "Block"

-- Performance Settings
-- @ui: slider(min=30, max=144, step=1) type=int
config.max_fps = 60

-- @ui: select(options="Software, Hardware, WebGpu") type=string
config.front_end = "OpenGL"

-- Advanced Features
-- @ui: select(options="Fancy, Retro, Modern") type=string
config.tab_bar_style = "Fancy"

-- @ui: slider(min=0.0, max=1.0, step=0.1) type=float
config.text_background_opacity = 1.0

-- <<TUNER-END>>

-- Additional configuration that won't appear in GUI
config.automatically_reload_config = true
config.check_for_updates = false

return config