//! Visibility cycling mirror of upstream `mindustry.ui.MultiReqImage`.

use super::ReqImage;

#[derive(Debug, Clone, PartialEq)]
pub struct MultiReqImage {
    displays: Vec<ReqImage>,
    time: f32,
}

impl Default for MultiReqImage {
    fn default() -> Self {
        Self {
            displays: Vec::new(),
            time: 0.0,
        }
    }
}

impl MultiReqImage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, display: ReqImage) {
        self.displays.push(display);
    }

    pub fn act(&mut self, delta: f32) {
        self.time += delta / 60.0;
        for display in &mut self.displays {
            display.visible = false;
        }

        if let Some(index) = self.displays.iter().position(ReqImage::valid) {
            self.displays[index].visible = true;
        } else if !self.displays.is_empty() {
            let index = self.time as usize % self.displays.len();
            self.displays[index].visible = true;
        }
    }

    pub fn displays(&self) -> &[ReqImage] {
        &self.displays
    }

    pub fn displays_mut(&mut self) -> &mut [ReqImage] {
        &mut self.displays
    }

    pub fn visible_index(&self) -> Option<usize> {
        self.displays.iter().position(|display| display.visible)
    }

    pub fn time(&self) -> f32 {
        self.time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_valid_req_image_is_visible_like_java_find() {
        let mut multi = MultiReqImage::new();
        multi.add(ReqImage::new("copper", false));
        multi.add(ReqImage::new("lead", true));
        multi.add(ReqImage::new("graphite", true));

        multi.act(1.0);

        assert_eq!(multi.visible_index(), Some(1));
        assert!(!multi.displays()[0].visible);
        assert!(multi.displays()[1].visible);
        assert!(!multi.displays()[2].visible);
    }

    #[test]
    fn invalid_images_cycle_by_integer_time_modulo_size() {
        let mut multi = MultiReqImage::new();
        multi.add(ReqImage::new("copper", false));
        multi.add(ReqImage::new("lead", false));

        multi.act(60.0);
        assert_eq!(multi.visible_index(), Some(1));
        multi.act(60.0);
        assert_eq!(multi.visible_index(), Some(0));
    }
}
