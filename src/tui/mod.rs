use crate::errors::AppError;

pub mod action;
pub mod controller;
pub mod library;
pub mod runtime;
pub mod state;
pub mod storage;
pub mod ui;

pub async fn run() -> Result<(), AppError> {
    runtime::run().await
}
