// Embedded web assets for the GUI interface

pub const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🎨 Wezztershier - WezTerm Configuration</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        
        .header {
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            padding: 1rem 2rem;
            border-bottom: 1px solid rgba(255, 255, 255, 0.2);
            color: white;
        }
        
        .header h1 {
            font-size: 1.5rem;
            margin-bottom: 0.25rem;
        }
        
        .content {
            flex: 1;
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 1px;
            background: rgba(255, 255, 255, 0.1);
        }
        
        .panel {
            background: rgba(255, 255, 255, 0.95);
            padding: 1rem;
            overflow-y: auto;
        }
        
        .widget-panel h2, .preview-panel h2 {
            margin-bottom: 1rem;
            color: #333;
        }
        
        .widget-item {
            margin-bottom: 1rem;
            padding: 1rem;
            background: #f5f5f5;
            border-radius: 8px;
            border-left: 4px solid #667eea;
        }
        
        .widget-label {
            font-weight: 600;
            margin-bottom: 0.5rem;
            color: #333;
        }
        
        .widget-input {
            width: 100%;
            padding: 0.5rem;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 14px;
        }
        
        .config-preview {
            background: #2d3748;
            color: #e2e8f0;
            padding: 1rem;
            border-radius: 8px;
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 14px;
            line-height: 1.6;
            white-space: pre-wrap;
        }
        
        .error {
            background: #fed7d7;
            color: #c53030;
            padding: 1rem;
            border-radius: 4px;
            margin-bottom: 1rem;
        }
        
        .loading {
            text-align: center;
            padding: 2rem;
            color: white;
        }
        
        .btn {
            background: #667eea;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
        }
        
        .btn:hover {
            background: #5a6fd8;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>🎨 Wezztershier - WezTerm Configuration</h1>
        <p>Beautiful GUI for configuring WezTerm settings</p>
    </div>
    
    <div id="app">
        <div class="loading">
            <h3>Loading configuration...</h3>
            <p>Please wait while we set up your interface</p>
        </div>
    </div>
    
    <script>
        class WezztershierApp {
            constructor() {
                this.widgets = {};
                this.configPreview = '';
                this.init();
            }
            
            async init() {
                try {
                    await this.loadSampleConfig();
                    this.render();
                } catch (error) {
                    this.showError('Failed to load configuration: ' + error.message);
                }
            }
            
            async loadSampleConfig() {
                const response = await fetch('/api/sample');
                const sampleConfig = await response.text();
                
                const parseResponse = await fetch('/api/parse', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ config_content: sampleConfig })
                });
                
                const result = await parseResponse.json();
                this.widgets = result.widgets;
                this.configPreview = result.config_preview;
            }
            
            async updateWidget(widgetId, value) {
                try {
                    const response = await fetch('/api/update', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ widget_id: widgetId, value })
                    });
                    
                    const result = await response.json();
                    this.widgets = result.widgets;
                    this.configPreview = result.config_preview;
                    this.render();
                } catch (error) {
                    this.showError('Failed to update widget: ' + error.message);
                }
            }
            
            render() {
                const app = document.getElementById('app');
                app.innerHTML = `
                    <div class="content">
                        <div class="panel widget-panel">
                            <h2>Configuration Widgets</h2>
                            ${this.renderWidgets()}
                        </div>
                        <div class="panel preview-panel">
                            <h2>Live Preview</h2>
                            <div class="config-preview">${this.configPreview || '-- Configuration will appear here --'}</div>
                        </div>
                    </div>
                `;
                
                this.attachEventListeners();
            }
            
            renderWidgets() {
                if (!this.widgets || Object.keys(this.widgets).length === 0) {
                    return '<p>No widgets found. Load a configuration file with @ui annotations.</p>';
                }
                
                return Object.entries(this.widgets).map(([id, widget]) => `
                    <div class="widget-item">
                        <div class="widget-label">${widget.config_key || id}</div>
                        ${this.renderWidget(id, widget)}
                    </div>
                `).join('');
            }
            
            renderWidget(id, widget) {
                const value = widget.current_value || '';
                
                if (widget.slider) {
                    const min = widget.slider.min || 0;
                    const max = widget.slider.max || 100;
                    const step = widget.slider.step || 1;
                    return `<input type="range" class="widget-input" data-widget="${id}" 
                               min="${min}" max="${max}" step="${step}" value="${value}">
                            <span>${value}</span>`;
                }
                
                if (widget.select) {
                    const options = (widget.select.options || []).map(opt => 
                        `<option value="${opt}" ${opt === value ? 'selected' : ''}>${opt}</option>`
                    ).join('');
                    return `<select class="widget-input" data-widget="${id}">${options}</select>`;
                }
                
                if (widget.color_picker) {
                    return `<input type="color" class="widget-input" data-widget="${id}" value="${value}">`;
                }
                
                return `<input type="text" class="widget-input" data-widget="${id}" value="${value}">`;
            }
            
            attachEventListeners() {
                document.querySelectorAll('[data-widget]').forEach(input => {
                    input.addEventListener('input', (e) => {
                        const widgetId = e.target.dataset.widget;
                        let value = e.target.value;
                        
                        // Convert to appropriate type
                        if (e.target.type === 'range' || e.target.type === 'number') {
                            value = parseFloat(value);
                        }
                        
                        this.updateWidget(widgetId, value);
                    });
                });
            }
            
            showError(message) {
                const app = document.getElementById('app');
                app.innerHTML = `<div class="error">Error: ${message}</div>`;
            }
        }
        
        // Initialize app when page loads
        document.addEventListener('DOMContentLoaded', () => {
            new WezztershierApp();
        });
    </script>
</body>
</html>"#;