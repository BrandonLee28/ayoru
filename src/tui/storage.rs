use crate::tui::library::LibraryState;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LibraryStorage {
    path: PathBuf,
}

impl LibraryStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<LibraryState, std::io::Error> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => serde_json::from_str(&contents)
                .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(LibraryState::default()),
            Err(err) => Err(err),
        }
    }

    pub fn save(&self, library: &LibraryState) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(library)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

        fs::write(&self.path, json)
    }
}
