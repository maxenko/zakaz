use plotters::coord::Shift;
use plotters::prelude::*;

use crate::error::AppError;
use crate::ib::types::HistoricalBar;
use super::theme::ChartTheme;
use super::types::{ChartViewport, VolumeBar};

pub struct CandlestickChart {
    width: u32,
    height: u32,
    theme: ChartTheme,
}

impl CandlestickChart {
    pub fn new(width: u32, height: u32, theme: ChartTheme) -> Self {
        Self { width, height, theme }
    }
    
    pub fn render_to_buffer(
        &self,
        bars: &[HistoricalBar],
        viewport: &ChartViewport,
    ) -> Result<Vec<u8>, AppError> {
        let mut buffer = vec![0u8; (self.width * self.height * 3) as usize];
        
        {
            let root = BitMapBackend::with_buffer(&mut buffer, (self.width, self.height))
                .into_drawing_area();
            
            // Fill background
            root.fill(&ChartTheme::parse_color(&self.theme.colors.background))?;
            
            // Calculate layout
            let main_height = (self.height as f64 * (1.0 - self.theme.volume_height_ratio)) as i32;
            let volume_height = (self.height as f64 * self.theme.volume_height_ratio) as i32;
            
            // Main price chart area
            let (upper, lower) = root.split_vertically(main_height);
            
            // Draw price chart
            self.draw_price_chart(&upper, bars, viewport)?;
            
            // Draw volume chart
            self.draw_volume_chart(&lower, bars, viewport)?;
            
            root.present()?;
        }
        
        Ok(buffer)
    }
    
    pub fn render_to_svg(
        &self,
        bars: &[HistoricalBar],
        viewport: &ChartViewport,
    ) -> Result<String, AppError> {
        let mut svg_string = String::new();
        
        {
            let root = SVGBackend::with_string(&mut svg_string, (self.width, self.height))
                .into_drawing_area();
            
            // Fill background
            root.fill(&ChartTheme::parse_color(&self.theme.colors.background))?;
            
            // Calculate layout
            let main_height = (self.height as f64 * (1.0 - self.theme.volume_height_ratio)) as i32;
            let volume_height = (self.height as f64 * self.theme.volume_height_ratio) as i32;
            
            // Main price chart area
            let (upper, lower) = root.split_vertically(main_height);
            
            // Draw price chart
            self.draw_price_chart(&upper, bars, viewport)?;
            
            // Draw volume chart
            self.draw_volume_chart(&lower, bars, viewport)?;
            
            root.present()?;
        }
        
        Ok(svg_string)
    }
    
    fn draw_price_chart<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, Shift>,
        bars: &[HistoricalBar],
        viewport: &ChartViewport,
    ) -> Result<(), AppError> 
    where 
        DB::ErrorType: 'static
    {
        // Apply padding
        let chart_area = area.margin(
            self.theme.padding.top as i32,
            self.theme.padding.bottom as i32,
            self.theme.padding.left as i32,
            self.theme.padding.right as i32,
        );
        
        // Create chart context
        let mut chart = ChartBuilder::on(&chart_area)
            .x_label_area_size(30)
            .y_label_area_size(50)
            .build_cartesian_2d(
                viewport.x_min..viewport.x_max,
                viewport.y_min..viewport.y_max,
            )?;
        
        // Draw grid
        chart.configure_mesh()
            .x_labels(10)
            .y_labels(10)
            .x_label_formatter(&|x| {
                let idx = (*x as usize).min(bars.len().saturating_sub(1));
                if idx < bars.len() {
                    bars[idx].timestamp.format("%m/%d").to_string()
                } else {
                    String::new()
                }
            })
            .y_label_formatter(&|y| format!("{:.2}", y))
            .axis_style(ChartTheme::parse_color(&self.theme.colors.axis_line))
            .label_style((
                self.theme.font_family.as_str(),
                self.theme.font_size,
                &ChartTheme::parse_color(&self.theme.colors.axis_text)
            ))
            .light_line_style(ChartTheme::parse_color(&self.theme.colors.grid_minor))
            .bold_line_style(ChartTheme::parse_color(&self.theme.colors.grid_major))
            .draw()?;
        
        // Calculate visible range
        let start_idx = viewport.x_min.floor().max(0.0) as usize;
        let end_idx = (viewport.x_max.ceil() as usize).min(bars.len());
        
        // Draw candlesticks
        for i in start_idx..end_idx {
            if i >= bars.len() {
                break;
            }
            
            let bar = &bars[i];
            let x = i as f64;
            
            // Calculate candle width
            let bar_width = 1.0 / (viewport.x_max - viewport.x_min) * chart_area.dim_in_pixel().0 as f64;
            let candle_width = bar_width * self.theme.candle_width_ratio;
            let half_width = candle_width / 2.0;
            
            let is_bullish = bar.close >= bar.open;
            let (body_color, wick_color) = if is_bullish {
                (
                    ChartTheme::parse_color(&self.theme.colors.candle_bullish_body),
                    ChartTheme::parse_color(&self.theme.colors.candle_bullish_wick),
                )
            } else {
                (
                    ChartTheme::parse_color(&self.theme.colors.candle_bearish_body),
                    ChartTheme::parse_color(&self.theme.colors.candle_bearish_wick),
                )
            };
            
            // Draw wick (high-low line)
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(x, bar.low), (x, bar.high)],
                wick_color.stroke_width(self.theme.wick_width as u32),
            )))?;
            
            // Draw body (open-close rectangle)
            let body_top = bar.open.max(bar.close);
            let body_bottom = bar.open.min(bar.close);
            
            if candle_width > 1.0 {
                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x - half_width / chart_area.dim_in_pixel().0 as f64, body_bottom), 
                     (x + half_width / chart_area.dim_in_pixel().0 as f64, body_top)],
                    body_color.filled(),
                )))?;
            }
        }
        
        Ok(())
    }
    
    fn draw_volume_chart<DB: DrawingBackend>(
        &self,
        area: &DrawingArea<DB, Shift>,
        bars: &[HistoricalBar],
        viewport: &ChartViewport,
    ) -> Result<(), AppError>
    where
        DB::ErrorType: 'static
    {
        // Apply padding
        let chart_area = area.margin(
            5,
            self.theme.padding.bottom as i32,
            self.theme.padding.left as i32,
            self.theme.padding.right as i32,
        );
        
        // Find max volume in visible range
        let start_idx = viewport.x_min.floor().max(0.0) as usize;
        let end_idx = (viewport.x_max.ceil() as usize).min(bars.len());
        
        let max_volume = bars[start_idx..end_idx]
            .iter()
            .map(|b| b.volume)
            .max()
            .unwrap_or(0) as f64;
        
        if max_volume == 0.0 {
            return Ok(());
        }
        
        // Create volume chart
        let mut chart = ChartBuilder::on(&chart_area)
            .y_label_area_size(50)
            .build_cartesian_2d(
                viewport.x_min..viewport.x_max,
                0.0..max_volume * 1.1,
            )?;
        
        // Configure volume chart
        chart.configure_mesh()
            .disable_x_mesh()
            .y_labels(3)
            .y_label_formatter(&|y| format!("{:.0}", y / 1000.0) + "K")
            .axis_style(ChartTheme::parse_color(&self.theme.colors.axis_line))
            .label_style((
                self.theme.font_family.as_str(),
                self.theme.font_size * 0.8,
                &ChartTheme::parse_color(&self.theme.colors.axis_text)
            ))
            .draw()?;
        
        // Draw volume bars
        for i in start_idx..end_idx {
            if i >= bars.len() {
                break;
            }
            
            let bar = &bars[i];
            let x = i as f64;
            let volume_bar = VolumeBar::from_historical_bar(bar);
            
            let bar_width = 1.0 / (viewport.x_max - viewport.x_min) * chart_area.dim_in_pixel().0 as f64;
            let volume_width = bar_width * self.theme.candle_width_ratio;
            let half_width = volume_width / 2.0;
            
            let color = if volume_bar.is_bullish {
                ChartTheme::parse_color(&self.theme.colors.volume_bullish)
            } else {
                ChartTheme::parse_color(&self.theme.colors.volume_bearish)
            };
            
            if volume_width > 1.0 {
                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x - half_width / chart_area.dim_in_pixel().0 as f64, 0.0),
                     (x + half_width / chart_area.dim_in_pixel().0 as f64, volume_bar.volume as f64)],
                    color.filled(),
                )))?;
            }
        }
        
        Ok(())
    }
}