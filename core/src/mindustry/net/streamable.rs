#[derive(Debug, Clone, PartialEq)]
pub struct Streamable {
    pub stream: Vec<u8>,
}

impl Streamable {
    pub fn new(stream: Vec<u8>) -> Self {
        Self { stream }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StreamBuilder {
    pub id: i32,
    pub packet_type: u8,
    pub total: i32,
    stream: Vec<u8>,
}

impl StreamBuilder {
    pub fn new(id: i32, packet_type: u8, total: i32) -> Self {
        Self {
            id,
            packet_type,
            total,
            stream: Vec::new(),
        }
    }

    pub fn progress(&self) -> f32 {
        if self.total <= 0 {
            1.0
        } else {
            self.stream.len() as f32 / self.total as f32
        }
    }

    pub fn add(&mut self, bytes: &[u8]) {
        self.stream.extend_from_slice(bytes);
    }

    pub fn is_done(&self) -> bool {
        self.stream.len() >= self.total.max(0) as usize
    }

    pub fn build(self) -> Streamable {
        Streamable::new(self.stream)
    }
}
