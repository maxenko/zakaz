use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("IB connection error: {0}")]
    IBConnection(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("{0}")]
    Custom(String),
    
    #[error("Chart rendering error: {0}")]
    ChartError(String),
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Custom(s)
    }
}

impl AppError {
    #[allow(dead_code)]
    pub fn custom(msg: impl Into<String>) -> Self {
        AppError::Custom(msg.into())
    }
}

impl<T> From<plotters::drawing::DrawingAreaErrorKind<T>> for AppError 
where 
    T: std::error::Error + Send + Sync + 'static
{
    fn from(err: plotters::drawing::DrawingAreaErrorKind<T>) -> Self {
        AppError::ChartError(format!("Drawing error: {:?}", err))
    }
}