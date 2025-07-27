use crate::ib::types::HistoricalBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ChartData {
    pub symbol: String,
    pub bars: Vec<HistoricalBar>,
    pub viewport: ChartViewport,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChartViewport {
    pub x_min: f64,  // Index of first visible bar
    pub x_max: f64,  // Index of last visible bar
    pub y_min: f64,  // Min price in viewport
    pub y_max: f64,  // Max price in viewport
}

impl ChartViewport {
    pub fn new() -> Self {
        Self {
            x_min: 0.0,
            x_max: 100.0,
            y_min: 0.0,
            y_max: 100.0,
        }
    }
    
    pub fn fit_to_data(bars: &[HistoricalBar], padding_percent: f64) -> Self {
        if bars.is_empty() {
            return Self::new();
        }
        
        let x_min = 0.0;
        let x_max = (bars.len() as f64) - 1.0;
        
        let mut y_min = f64::MAX;
        let mut y_max = f64::MIN;
        
        for bar in bars {
            y_min = y_min.min(bar.low);
            y_max = y_max.max(bar.high);
        }
        
        // Add padding
        let y_padding = (y_max - y_min) * padding_percent / 100.0;
        
        Self {
            x_min,
            x_max,
            y_min: y_min - y_padding,
            y_max: y_max + y_padding,
        }
    }
    
    pub fn zoom(&mut self, factor: f64, center_x: f64, center_y: f64) {
        // Calculate current spans
        let x_span = self.x_max - self.x_min;
        let y_span = self.y_max - self.y_min;
        
        // Calculate new spans
        let new_x_span = x_span / factor;
        let new_y_span = y_span / factor;
        
        // Calculate ratios for centering
        let x_ratio = (center_x - self.x_min) / x_span;
        let y_ratio = (center_y - self.y_min) / y_span;
        
        // Update viewport
        self.x_min = center_x - new_x_span * x_ratio;
        self.x_max = center_x + new_x_span * (1.0 - x_ratio);
        self.y_min = center_y - new_y_span * y_ratio;
        self.y_max = center_y + new_y_span * (1.0 - y_ratio);
    }
    
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.x_min += dx;
        self.x_max += dx;
        self.y_min += dy;
        self.y_max += dy;
    }
    
    pub fn constrain_to_data(&mut self, data_length: usize, min_y: f64, max_y: f64) {
        // Constrain X axis
        let x_span = self.x_max - self.x_min;
        let max_x = (data_length as f64) - 1.0;
        
        if self.x_min < 0.0 {
            self.x_min = 0.0;
            self.x_max = x_span.min(max_x);
        }
        if self.x_max > max_x {
            self.x_max = max_x;
            self.x_min = (max_x - x_span).max(0.0);
        }
        
        // Ensure minimum span
        if self.x_max - self.x_min < 5.0 {
            self.x_max = self.x_min + 5.0;
        }
        
        // Constrain Y axis with padding
        let y_padding = (max_y - min_y) * 0.1;
        let constrained_min_y = min_y - y_padding;
        let constrained_max_y = max_y + y_padding;
        
        if self.y_min < constrained_min_y {
            self.y_min = constrained_min_y;
        }
        if self.y_max > constrained_max_y {
            self.y_max = constrained_max_y;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChartInteraction {
    pub is_panning: bool,
    pub last_mouse_x: f64,
    pub last_mouse_y: f64,
    pub chart_width: u32,
    pub chart_height: u32,
}

impl ChartInteraction {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            is_panning: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            chart_width: width,
            chart_height: height,
        }
    }
    
    pub fn screen_to_chart(&self, screen_x: f64, screen_y: f64, viewport: &ChartViewport) -> (f64, f64) {
        let chart_x = viewport.x_min + (screen_x / self.chart_width as f64) * (viewport.x_max - viewport.x_min);
        let chart_y = viewport.y_max - (screen_y / self.chart_height as f64) * (viewport.y_max - viewport.y_min);
        (chart_x, chart_y)
    }
    
    pub fn calculate_pan_delta(&self, current_x: f64, current_y: f64, viewport: &ChartViewport) -> (f64, f64) {
        let dx_screen = current_x - self.last_mouse_x;
        let dy_screen = current_y - self.last_mouse_y;
        
        let dx_chart = -dx_screen * (viewport.x_max - viewport.x_min) / self.chart_width as f64;
        let dy_chart = dy_screen * (viewport.y_max - viewport.y_min) / self.chart_height as f64;
        
        (dx_chart, dy_chart)
    }
}

#[derive(Debug, Clone)]
pub struct VolumeBar {
    pub volume: i64,
    pub is_bullish: bool,  // true if close > open
}

impl VolumeBar {
    pub fn from_historical_bar(bar: &HistoricalBar) -> Self {
        Self {
            volume: bar.volume,
            is_bullish: bar.close >= bar.open,
        }
    }
}