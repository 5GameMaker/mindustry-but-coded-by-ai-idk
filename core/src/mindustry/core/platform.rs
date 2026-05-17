#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformInfo {
    pub name: String,
    pub headless: bool,
}

impl PlatformInfo {
    pub fn headless() -> Self {
        Self {
            name: "headless".into(),
            headless: true,
        }
    }

    pub fn desktop() -> Self {
        Self {
            name: "desktop".into(),
            headless: false,
        }
    }
}
