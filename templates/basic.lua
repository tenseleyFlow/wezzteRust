-- Basic WezTerm configuration with Wezzte annotations
-- Copy this to ~/.config/wezterm/wezterm.lua and customize

local wezterm = require('wezterm')
local config = {}

-- <<TUNER-START>>

-- Font Configuration
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 12

-- Theme Selection  
-- @ui: theme_selector(themes="builtin") type=string
config.color_scheme = "Dracula"

-- Window Settings
-- @ui: slider(min=0.1, max=1.0, step=0.05) type=float
config.window_background_opacity = 1.0

-- @ui: select(options="bottom, top, hidden") type=string
config.tab_bar_at_bottom = "bottom"

-- Cursor Style
-- @ui: select(options="Block, Underline, Bar") type=string
config.default_cursor_style = "Block"

-- <<TUNER-END>>

return config