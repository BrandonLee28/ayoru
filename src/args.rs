use clap::Parser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args;

#[derive(Parser, Debug)]
#[command(
    name = "ayoru",
    version,
    about = "A quieter way to watch anime.",
    override_usage = "ayoru",
    after_help = "Examples:\n  ayoru"
)]
struct RawArgs;

pub fn parse_from<I, T>(itr: I) -> Result<Args, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    RawArgs::try_parse_from(itr)?;
    Ok(Args)
}
