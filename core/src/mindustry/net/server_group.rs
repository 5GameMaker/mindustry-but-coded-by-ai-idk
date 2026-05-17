#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerGroup {
    pub name: String,
    pub addresses: Vec<String>,
    pub prioritized: bool,
    pub hidden: bool,
    pub favorite: bool,
}

impl ServerGroup {
    pub fn new(name: impl Into<String>, addresses: Vec<String>, prioritized: bool) -> Self {
        Self {
            name: name.into(),
            addresses,
            prioritized,
            hidden: false,
            favorite: false,
        }
    }

    pub fn key(&self) -> String {
        let suffix = if self.name.is_empty() {
            self.addresses.first().cloned().unwrap_or_default()
        } else {
            self.name.clone()
        };
        format!("server-{suffix}")
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
    }

    pub fn set_favorite(&mut self, favorite: bool) {
        self.favorite = favorite;
    }
}

impl Default for ServerGroup {
    fn default() -> Self {
        Self::new("", Vec::new(), false)
    }
}
