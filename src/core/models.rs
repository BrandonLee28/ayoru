#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Episode {
    pub number: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamCandidate {
    pub provider: String,
    pub url: String,
    pub is_sub: bool,
    pub resolution: Option<u16>,
}
