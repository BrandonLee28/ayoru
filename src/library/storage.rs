use crate::library::LibraryState;
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

        // Write beside the destination, then rename so an interrupted save keeps the old file.
        let temporary = self
            .path
            .with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temporary, json)?;
        fs::rename(temporary, &self.path)
    }
}

pub fn default_library_path() -> PathBuf {
    if let Some(root) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(root).join("ayoru/library.json");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local/state/ayoru/library.json");
    }
    PathBuf::from(".ayoru/library.json")
}
