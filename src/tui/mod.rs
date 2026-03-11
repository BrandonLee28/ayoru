use crate::errors::AppError;

pub mod action;
pub mod controller;
pub mod runtime;
pub mod state;

pub async fn run() -> Result<(), AppError> {
    runtime::run().await
}
