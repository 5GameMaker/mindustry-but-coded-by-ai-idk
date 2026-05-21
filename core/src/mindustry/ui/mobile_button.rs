//! Mobile image button shell mirroring upstream `mindustry.ui.MobileButton`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobileButtonLayout {
    pub grow_x: bool,
    pub wrap: bool,
    pub center_x: bool,
    pub center_y: bool,
    pub row_after_icon: bool,
}

impl Default for MobileButtonLayout {
    fn default() -> Self {
        Self {
            grow_x: true,
            wrap: true,
            center_x: true,
            center_y: true,
            row_after_icon: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileButton {
    pub icon: String,
    pub text: String,
    pub layout: MobileButtonLayout,
    clicks: usize,
}

impl MobileButton {
    pub fn new(icon: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            text: text.into(),
            layout: MobileButtonLayout::default(),
            clicks: 0,
        }
    }

    pub fn click(&mut self, mut listener: impl FnMut()) {
        self.clicks += 1;
        listener();
    }

    pub fn clicks(&self) -> usize {
        self.clicks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_button_keeps_icon_text_and_java_layout_defaults() {
        let button = MobileButton::new("settings", "@settings");

        assert_eq!(button.icon, "settings");
        assert_eq!(button.text, "@settings");
        assert_eq!(
            button.layout,
            MobileButtonLayout {
                grow_x: true,
                wrap: true,
                center_x: true,
                center_y: true,
                row_after_icon: true
            }
        );
    }

    #[test]
    fn click_invokes_listener_and_records_click_count() {
        let mut button = MobileButton::new("play", "@play");
        let mut calls = 0;

        button.click(|| calls += 1);
        button.click(|| calls += 1);

        assert_eq!(calls, 2);
        assert_eq!(button.clicks(), 2);
    }
}
