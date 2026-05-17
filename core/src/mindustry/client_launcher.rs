use crate::mindustry::vars::AppContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLauncher {
    pub context: AppContext,
    pub loaded: bool,
}

impl ClientLauncher {
    pub fn new(context: AppContext) -> Self {
        Self {
            context,
            loaded: false,
        }
    }

    pub fn setup(&mut self) {
        self.loaded = false;
    }

    pub fn update(&mut self) {
        self.loaded = true;
    }
}
