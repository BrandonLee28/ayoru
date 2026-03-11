use crate::errors::AppError;

pub mod runtime;

pub async fn run() -> Result<(), AppError> {
    runtime::run().await
}
