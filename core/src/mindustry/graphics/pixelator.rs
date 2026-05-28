use super::Layer;

/// Lightweight camera snapshot used by the Rust pixelation plan layer.
///
/// Upstream `Pixelator` mutates `Core.camera` and `renderer.scale` before the
/// actual world draw, then restores both on `Layer.end`.  The Rust port keeps
/// that behavior as explicit data so desktop/mobile backends can apply it
/// without coupling `mindustry-core` to a GPU API.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelatorCamera {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PixelatorCamera {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelatorInput {
    pub enabled: bool,
    pub renderer_scale: f32,
    pub land_scale: f32,
    pub cutscene: bool,
    pub graphics_width: i32,
    pub graphics_height: i32,
    pub camera: PixelatorCamera,
}

impl PixelatorInput {
    pub const fn new(
        enabled: bool,
        renderer_scale: f32,
        land_scale: f32,
        cutscene: bool,
        graphics_width: i32,
        graphics_height: i32,
        camera: PixelatorCamera,
    ) -> Self {
        Self {
            enabled,
            renderer_scale,
            land_scale,
            cutscene,
            graphics_width,
            graphics_height,
            camera,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelatorRestorePlan {
    pub layer: f32,
    pub camera_x: f32,
    pub camera_y: f32,
    pub renderer_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelatorFramePlan {
    pub buffer_width: i32,
    pub buffer_height: i32,
    pub snapped_camera_x: f32,
    pub snapped_camera_y: f32,
    pub pixel_scale: f32,
    pub previous_scale: f32,
    pub clear_color_rgba: [f32; 4],
    pub restore: PixelatorRestorePlan,
}

impl PixelatorFramePlan {
    pub const fn restore_plan(&self) -> PixelatorRestorePlan {
        self.restore
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelatorState {
    pub last_camera_x: f32,
    pub last_camera_y: f32,
    pub previous_scale: f32,
    pub last_buffer_width: i32,
    pub last_buffer_height: i32,
}

impl Default for PixelatorState {
    fn default() -> Self {
        Self {
            last_camera_x: 0.0,
            last_camera_y: 0.0,
            previous_scale: 1.0,
            last_buffer_width: 2,
            last_buffer_height: 2,
        }
    }
}

impl PixelatorState {
    pub fn enabled(input: &PixelatorInput) -> bool {
        input.enabled
    }

    pub fn draw_pixelate_plan(&mut self, input: PixelatorInput) -> Option<PixelatorFramePlan> {
        if !Self::enabled(&input) {
            return None;
        }

        let previous_scale = input.renderer_scale;
        let pixel_scale = input.renderer_scale.trunc();
        let pixel_scale = if pixel_scale.is_finite() && pixel_scale > 0.0 {
            pixel_scale
        } else {
            1.0
        };

        let camera_width = input.camera.width.trunc() as i32;
        let camera_height = input.camera.height.trunc() as i32;
        let snapped_camera_x = input.camera.x.trunc()
            + if camera_width.rem_euclid(2) == 0 {
                0.0
            } else {
                0.5
            };
        let snapped_camera_y = input.camera.y.trunc()
            + if camera_height.rem_euclid(2) == 0 {
                0.0
            } else {
                0.5
            };

        let mut buffer_width = camera_width;
        let mut buffer_height = camera_height;
        if input.cutscene {
            buffer_width = (input.camera.width * input.land_scale / pixel_scale).trunc() as i32;
            buffer_height = (input.camera.height * input.land_scale / pixel_scale).trunc() as i32;
        }

        buffer_width = clamp_i32(buffer_width, 2, input.graphics_width.max(2));
        buffer_height = clamp_i32(buffer_height, 2, input.graphics_height.max(2));

        self.last_camera_x = input.camera.x;
        self.last_camera_y = input.camera.y;
        self.previous_scale = previous_scale;
        self.last_buffer_width = buffer_width;
        self.last_buffer_height = buffer_height;

        Some(PixelatorFramePlan {
            buffer_width,
            buffer_height,
            snapped_camera_x,
            snapped_camera_y,
            pixel_scale,
            previous_scale,
            clear_color_rgba: [0.0, 0.0, 0.0, 0.0],
            restore: PixelatorRestorePlan {
                layer: Layer::END,
                camera_x: input.camera.x,
                camera_y: input.camera.y,
                renderer_scale: previous_scale,
            },
        })
    }
}

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixelator_disabled_returns_no_plan() {
        let mut state = PixelatorState::default();
        let input = PixelatorInput::new(
            false,
            4.8,
            1.0,
            false,
            1920,
            1080,
            PixelatorCamera::new(12.4, 24.9, 801.9, 600.2),
        );

        assert_eq!(state.draw_pixelate_plan(input), None);
    }

    #[test]
    fn pixelator_plan_matches_java_snap_and_clamp_rules() {
        let mut state = PixelatorState::default();
        let input = PixelatorInput::new(
            true,
            4.8,
            1.0,
            false,
            640,
            480,
            PixelatorCamera::new(12.4, 24.9, 801.9, 599.2),
        );

        let plan = state.draw_pixelate_plan(input).unwrap();
        assert_eq!(plan.pixel_scale, 4.0);
        assert_eq!(plan.buffer_width, 640);
        assert_eq!(plan.buffer_height, 480);
        assert_eq!(plan.snapped_camera_x, 12.5);
        assert_eq!(plan.snapped_camera_y, 24.5);
        assert_eq!(plan.restore.layer, Layer::END);
        assert_eq!(plan.restore.camera_x, 12.4);
        assert_eq!(plan.restore.camera_y, 24.9);
        assert_eq!(plan.restore.renderer_scale, 4.8);
    }

    #[test]
    fn pixelator_cutscene_uses_land_scale_over_pixel_scale() {
        let mut state = PixelatorState::default();
        let input = PixelatorInput::new(
            true,
            3.25,
            2.0,
            true,
            1000,
            900,
            PixelatorCamera::new(1.0, 2.0, 300.0, 150.0),
        );

        let plan = state.draw_pixelate_plan(input).unwrap();
        assert_eq!(plan.pixel_scale, 3.0);
        assert_eq!(plan.buffer_width, 200);
        assert_eq!(plan.buffer_height, 100);
    }

    #[test]
    fn pixelator_frame_plan_exposes_restore_without_becoming_draw_pass() {
        let mut state = PixelatorState::default();
        let input = PixelatorInput::new(
            true,
            2.0,
            1.0,
            false,
            800,
            600,
            PixelatorCamera::new(10.2, 20.7, 320.0, 240.0),
        );

        let plan = state.draw_pixelate_plan(input).unwrap();
        let restore = plan.restore_plan();

        assert_eq!(restore.layer, Layer::END);
        assert_eq!(restore.camera_x, 10.2);
        assert_eq!(restore.camera_y, 20.7);
        assert_eq!(restore.renderer_scale, 2.0);
    }
}
