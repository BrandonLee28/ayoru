#[tokio::main]
async fn main() {
    let args = match ani::args::parse_from(std::env::args_os()) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };

    match args.command {
        ani::args::Command::Tui => {
            if let Err(err) = ani::tui::run().await {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
        ani::args::Command::Play { query } => {
            let query = query.join(" ");
            let provider = ani::provider::allanime::AllAnimeProvider::new();
            let runtime = ani::app::SystemPlayerRuntime;
            let picker = ani::cli::interactive::InteractivePickerRuntime;

            if let Err(err) = ani::app::run_with(&query, &provider, &runtime, &picker).await {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
    }
}
