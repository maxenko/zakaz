use super::types::ChartViewport;

#[derive(Debug)]
pub struct ViewportController {
    viewport: ChartViewport,
    min_zoom_bars: f64,
    max_zoom_bars: f64,
    data_length: usize,
}

impl ViewportController {
    pub fn new(data_length: usize) -> Self {
        let viewport = if data_length > 0 {
            ChartViewport {
                x_min: 0.0,
                x_max: (data_length as f64 - 1.0).min(100.0),
                y_min: 0.0,
                y_max: 100.0,
            }
        } else {
            ChartViewport::new()
        };
        
        Self {
            viewport,
            min_zoom_bars: 5.0,
            max_zoom_bars: 500.0,
            data_length,
        }
    }
    
    pub fn update_data_length(&mut self, new_length: usize) {
        self.data_length = new_length;
        if self.viewport.x_max > new_length as f64 - 1.0 {
            self.viewport.x_max = (new_length as f64 - 1.0).max(0.0);
        }
    }
    
    pub fn zoom(&mut self, factor: f64, center_x: f64, center_y: f64) {
        let current_bars = self.viewport.x_max - self.viewport.x_min;
        let new_bars = current_bars / factor;
        
        // Apply zoom limits
        if new_bars < self.min_zoom_bars || new_bars > self.max_zoom_bars {
            return;
        }
        
        self.viewport.zoom(factor, center_x, center_y);
        self.constrain_viewport();
    }
    
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.viewport.pan(dx, dy);
        self.constrain_viewport();
    }
    
    pub fn reset_zoom(&mut self) {
        if self.data_length > 0 {
            self.viewport.x_min = 0.0;
            self.viewport.x_max = (self.data_length as f64 - 1.0).min(100.0);
        }
    }
    
    pub fn fit_y_axis(&mut self, visible_y_min: f64, visible_y_max: f64) {
        let padding = (visible_y_max - visible_y_min) * 0.1;
        self.viewport.y_min = visible_y_min - padding;
        self.viewport.y_max = visible_y_max + padding;
    }
    
    pub fn get_visible_bar_range(&self) -> (usize, usize) {
        let start = self.viewport.x_min.floor().max(0.0) as usize;
        let end = (self.viewport.x_max.ceil() as usize).min(self.data_length.saturating_sub(1));
        (start, end)
    }
    
    pub fn get_viewport(&self) -> ChartViewport {
        self.viewport
    }
    
    pub fn set_viewport(&mut self, viewport: ChartViewport) {
        self.viewport = viewport;
        self.constrain_viewport();
    }
    
    fn constrain_viewport(&mut self) {
        if self.data_length == 0 {
            return;
        }
        
        let max_x = (self.data_length as f64) - 1.0;
        
        // Constrain zoom
        let x_span = self.viewport.x_max - self.viewport.x_min;
        if x_span < self.min_zoom_bars {
            let center = (self.viewport.x_min + self.viewport.x_max) / 2.0;
            self.viewport.x_min = center - self.min_zoom_bars / 2.0;
            self.viewport.x_max = center + self.min_zoom_bars / 2.0;
        } else if x_span > self.max_zoom_bars {
            let center = (self.viewport.x_min + self.viewport.x_max) / 2.0;
            self.viewport.x_min = center - self.max_zoom_bars / 2.0;
            self.viewport.x_max = center + self.max_zoom_bars / 2.0;
        }
        
        // Constrain pan
        if self.viewport.x_min < 0.0 {
            let shift = -self.viewport.x_min;
            self.viewport.x_min = 0.0;
            self.viewport.x_max += shift;
        }
        if self.viewport.x_max > max_x {
            let shift = self.viewport.x_max - max_x;
            self.viewport.x_max = max_x;
            self.viewport.x_min = (self.viewport.x_min - shift).max(0.0);
        }
    }
}