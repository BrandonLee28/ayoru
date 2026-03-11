#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamCandidate {
    pub provider: String,
    pub url: String,
    pub is_sub: bool,
    pub resolution: Option<u16>,
}
