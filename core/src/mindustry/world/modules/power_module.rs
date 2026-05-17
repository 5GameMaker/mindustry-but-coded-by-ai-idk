#[derive(Debug, Clone, PartialEq, Default)]
pub struct PowerModule {
    pub status: f32,
    pub init: bool,
    pub links: Vec<i32>,
}

impl PowerModule {
    pub fn sanitize(&mut self) {
        if self.status.is_nan() || self.status.is_infinite() {
            self.status = 0.0;
        }
    }
}
