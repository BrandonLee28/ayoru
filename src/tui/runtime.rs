use crate::errors::AppError;

pub trait TerminalSession {
    fn enable_raw_mode(&mut self) -> std::io::Result<()>;
    fn disable_raw_mode(&mut self) -> std::io::Result<()>;
    fn enter_alt_screen(&mut self) -> std::io::Result<()>;
    fn leave_alt_screen(&mut self) -> std::io::Result<()>;
}

#[async_trait::async_trait]
pub trait RuntimeApp {
    async fn step(&mut self) -> Result<RunDecision, AppError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunDecision {
    Continue,
    Quit,
}

pub async fn run_with_terminal<T, A>(terminal: &mut T, mut app: A) -> Result<(), AppError>
where
    T: TerminalSession,
    A: RuntimeApp,
{
    terminal
        .enable_raw_mode()
        .map_err(|e| AppError::Provider(e.to_string()))?;
    terminal
        .enter_alt_screen()
        .map_err(|e| AppError::Provider(e.to_string()))?;

    let result = app.step().await;

    terminal
        .leave_alt_screen()
        .map_err(|e| AppError::Provider(e.to_string()))?;
    terminal
        .disable_raw_mode()
        .map_err(|e| AppError::Provider(e.to_string()))?;

    match result? {
        RunDecision::Continue | RunDecision::Quit => Ok(()),
    }
}

pub async fn run() -> Result<(), AppError> {
    let mut terminal = CrosstermTerminalSession;
    run_with_terminal(&mut terminal, ImmediateQuit).await
}

struct CrosstermTerminalSession;

impl TerminalSession for CrosstermTerminalSession {
    fn enable_raw_mode(&mut self) -> std::io::Result<()> {
        crossterm::terminal::enable_raw_mode()
    }

    fn disable_raw_mode(&mut self) -> std::io::Result<()> {
        crossterm::terminal::disable_raw_mode()
    }

    fn enter_alt_screen(&mut self) -> std::io::Result<()> {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::EnterAlternateScreen
        )
    }

    fn leave_alt_screen(&mut self) -> std::io::Result<()> {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen
        )
    }
}

struct ImmediateQuit;

#[async_trait::async_trait]
impl RuntimeApp for ImmediateQuit {
    async fn step(&mut self) -> Result<RunDecision, AppError> {
        Ok(RunDecision::Quit)
    }
}
