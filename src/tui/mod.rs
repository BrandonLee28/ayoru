use crate::errors::AppError;

pub async fn run() -> Result<(), AppError> {
    Err(AppError::Provider(
        "TUI runtime is not implemented yet".to_string(),
    ))
}
