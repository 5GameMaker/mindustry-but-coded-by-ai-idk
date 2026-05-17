#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub id: usize,
    pub name: String,
}

impl Attribute {
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    pub fn vanilla() -> Vec<Self> {
        ["heat", "spores", "water", "oil", "light", "sand", "steam"]
            .into_iter()
            .enumerate()
            .map(|(id, name)| Self::new(id, name))
            .collect()
    }
}
