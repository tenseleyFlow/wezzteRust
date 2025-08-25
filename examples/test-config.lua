-- Example WezTerm configuration with Wezzte annotations
-- This demonstrates the advanced features of Phase 4

local wezterm = require('wezterm')
local config = {}

-- <<TUNER-START>>

-- Font Configuration Section
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 14

-- Theme and Color Configuration
-- @ui: theme_selector(themes="builtin", filter="all") type=string
config.color_scheme = "dracula"

-- @ui: color_picker(format="hex", alpha=false) type=color
config.colors.background = "#282a36"

-- @ui: color_picker(format="rgba", alpha=true) type=color  
config.colors.foreground = "rgba(248, 248, 242, 1.0)"

-- @ui: color_picker(format="hsl", alpha=false) type=color
config.colors.cursor_bg = "hsl(250, 100%, 80%)"

-- Terminal Behavior
-- @ui: select(options="bottom, top, hidden") type=string
config.tab_bar_at_bottom = "bottom"

-- @ui: select(options="Block, Underline, Bar") type=string
config.default_cursor_style = "Block"

-- @ui: slider(min=0.1, max=2.0, step=0.1) type=float
config.window_background_opacity = 0.95

-- Advanced Features
-- @ui: select(options="Fancy, Retro, Modern") type=string
config.tab_bar_style = "Fancy"

-- @ui: slider(min=0, max=100, step=5) type=int
config.max_fps = 60

-- @ui: color_picker(format="hex", alpha=true) type=color
config.colors.tab_bar = "#1e1e2e"

-- <<TUNER-END>>

return config