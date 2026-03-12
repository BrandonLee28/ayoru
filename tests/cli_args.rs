#[test]
fn accepts_bare_command_only() {
    ayoru::args::parse_from(["ayoru"]).unwrap();
}

#[test]
fn rejects_query_arguments() {
    let err = ayoru::args::parse_from(["ayoru", "frieren"]);
    assert!(err.is_err());
}

#[test]
fn rejects_legacy_tui_subcommand() {
    let err = ayoru::args::parse_from(["ayoru", "tui"]);
    assert!(err.is_err());
}
