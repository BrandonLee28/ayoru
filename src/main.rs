#[tokio::main]
async fn main() {
    match ayoru::args::parse_from(std::env::args_os()) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    }

    if let Err(err) = ayoru::tui::run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
