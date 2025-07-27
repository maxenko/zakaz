use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartColors {
    // Background
    pub background: String,
    pub grid_major: String,
    pub grid_minor: String,
    
    // Candles
    pub candle_bullish_body: String,
    pub candle_bullish_wick: String,
    pub candle_bearish_body: String,
    pub candle_bearish_wick: String,
    
    // Volume
    pub volume_bullish: String,
    pub volume_bearish: String,
    
    // Axes
    pub axis_text: String,
    pub axis_line: String,
    
    // Crosshair
    pub crosshair: String,
    pub crosshair_text_bg: String,
    pub crosshair_text: String,
    
    // Indicators
    pub atr_line: String,
    pub ma_line: String,
}

impl Default for ChartColors {
    fn default() -> Self {
        Self {
            // Dark theme by default
            background: "#1a1a1a".to_string(),
            grid_major: "#333333".to_string(),
            grid_minor: "#262626".to_string(),
            
            candle_bullish_body: "#26a69a".to_string(),
            candle_bullish_wick: "#26a69a".to_string(),
            candle_bearish_body: "#ef5350".to_string(),
            candle_bearish_wick: "#ef5350".to_string(),
            
            volume_bullish: "#26a69a80".to_string(),  // 50% opacity
            volume_bearish: "#ef535080".to_string(),  // 50% opacity
            
            axis_text: "#cccccc".to_string(),
            axis_line: "#666666".to_string(),
            
            crosshair: "#ffffff66".to_string(),  // 40% opacity
            crosshair_text_bg: "#000000cc".to_string(),  // 80% opacity
            crosshair_text: "#ffffff".to_string(),
            
            atr_line: "#ff9800".to_string(),
            ma_line: "#2196f3".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartTheme {
    pub colors: ChartColors,
    pub font_family: String,
    pub font_size: f64,
    pub candle_width_ratio: f64,  // 0.0 to 1.0, portion of bar width
    pub wick_width: f64,
    pub volume_height_ratio: f64,  // Portion of chart height for volume
    pub padding: ChartPadding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPadding {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Default for ChartTheme {
    fn default() -> Self {
        Self {
            colors: ChartColors::default(),
            font_family: "Arial".to_string(),
            font_size: 12.0,
            candle_width_ratio: 0.8,
            wick_width: 1.0,
            volume_height_ratio: 0.2,
            padding: ChartPadding {
                top: 20.0,
                right: 60.0,
                bottom: 40.0,
                left: 10.0,
            },
        }
    }
}

impl ChartTheme {
    pub fn light() -> Self {
        let mut theme = Self::default();
        theme.colors = ChartColors {
            background: "#ffffff".to_string(),
            grid_major: "#e0e0e0".to_string(),
            grid_minor: "#f0f0f0".to_string(),
            
            candle_bullish_body: "#4caf50".to_string(),
            candle_bullish_wick: "#4caf50".to_string(),
            candle_bearish_body: "#f44336".to_string(),
            candle_bearish_wick: "#f44336".to_string(),
            
            volume_bullish: "#4caf5080".to_string(),
            volume_bearish: "#f4433680".to_string(),
            
            axis_text: "#333333".to_string(),
            axis_line: "#999999".to_string(),
            
            crosshair: "#00000066".to_string(),
            crosshair_text_bg: "#ffffffcc".to_string(),
            crosshair_text: "#000000".to_string(),
            
            atr_line: "#ff6f00".to_string(),
            ma_line: "#1976d2".to_string(),
        };
        theme
    }
    
    pub fn parse_color(color: &str) -> plotters::style::RGBAColor {
        if color.starts_with('#') && color.len() >= 7 {
            let r = u8::from_str_radix(&color[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&color[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&color[5..7], 16).unwrap_or(0);
            let a = if color.len() >= 9 {
                u8::from_str_radix(&color[7..9], 16).unwrap_or(255)
            } else {
                255
            };
            plotters::style::RGBAColor(r, g, b, a as f64 / 255.0)
        } else {
            plotters::style::RGBAColor(0, 0, 0, 1.0)
        }
    }
}