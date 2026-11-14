fn main() {
    if let Err(err) = ayoru::args::parse_from(std::env::args_os()) {
        err.exit();
    }
    #[cfg(target_os = "macos")]
    ayoru::desktop::run();
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("Ayoru Desktop currently supports macOS only.");
        std::process::exit(1);
    }
}
