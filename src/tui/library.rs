#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SavedTitle {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SavedWatch {
    pub title: SavedTitle,
    pub episode: u32,
    pub watched_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct LibraryState {
    pub favorites: Vec<SavedTitle>,
    pub history: Vec<SavedWatch>,
    pub recently_watched: Vec<SavedWatch>,
}

impl LibraryState {
    pub fn toggle_favorite(&mut self, title: SavedTitle) {
        if let Some(index) = self.favorites.iter().position(|item| item.id == title.id) {
            self.favorites.remove(index);
        } else {
            self.favorites.push(title);
        }
    }

    pub fn record_watch(&mut self, watch: SavedWatch) {
        self.history.push(watch.clone());

        if let Some(index) = self
            .recently_watched
            .iter()
            .position(|item| item.title.id == watch.title.id)
        {
            self.recently_watched.remove(index);
        }

        self.recently_watched.insert(0, watch);
    }

    pub fn remove_favorite_at(&mut self, index: usize) {
        if index < self.favorites.len() {
            self.favorites.remove(index);
        }
    }

    pub fn remove_history_at(&mut self, index: usize) {
        if index < self.history.len() {
            self.history.remove(index);
        }
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        self.recently_watched.clear();
    }
}
