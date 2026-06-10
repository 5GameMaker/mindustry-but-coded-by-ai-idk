//! Canvas editing dialog model mirroring upstream
//! `mindustry.ui.dialogs.CanvasEditDialog`.

use crate::mindustry::world::blocks::logic::{
    canvas_bits_per_pixel, canvas_data_len, canvas_get_pixel, canvas_set_pixel, CanvasBlockState,
};

pub const CANVAS_EDIT_REFRESH_TIME: f32 = 60.0 * 2.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasEditSpec {
    pub canvas_size: i32,
    pub palette: Vec<u32>,
}

impl CanvasEditSpec {
    pub fn new(canvas_size: i32, palette: Vec<u32>) -> Self {
        Self {
            canvas_size,
            palette,
        }
    }

    pub fn bits_per_pixel(&self) -> i32 {
        canvas_bits_per_pixel(self.palette.len())
    }

    pub fn data_len(&self) -> usize {
        canvas_data_len(self.canvas_size, self.bits_per_pixel())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasPointerButton {
    Left,
    Middle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasEditAction {
    Hide,
    Configure(CanvasBlockState),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasGridLine {
    pub start: (f32, f32),
    pub end: (f32, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasDrawPlan {
    pub grid: bool,
    pub cell_width: f32,
    pub cell_height: f32,
    pub grid_lines: Vec<CanvasGridLine>,
    pub hover_cell: Option<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasEditDialog {
    spec: CanvasEditSpec,
    pixels: Vec<u32>,
    cur_color: u32,
    fill: bool,
    modified: bool,
    grid: bool,
    time: f32,
    last: Option<(i32, i32)>,
}

impl CanvasEditDialog {
    pub fn new(spec: CanvasEditSpec, state: &CanvasBlockState) -> Self {
        let bits_per_pixel = spec.bits_per_pixel();
        let pixels = (0..spec.canvas_size * spec.canvas_size)
            .map(|pos| {
                let index = canvas_get_pixel(&state.data, spec.canvas_size, bits_per_pixel, pos);
                spec.palette[index as usize]
            })
            .collect::<Vec<_>>();

        let cur_color = spec.palette[0];
        Self {
            spec,
            pixels,
            cur_color,
            fill: false,
            modified: false,
            grid: true,
            time: 0.0,
            last: None,
        }
    }

    pub fn new_blank(spec: CanvasEditSpec) -> Self {
        let state = CanvasBlockState::new(spec.data_len());
        Self::new(spec, &state)
    }

    pub fn selected_color(&self) -> u32 {
        self.cur_color
    }

    pub fn fill_enabled(&self) -> bool {
        self.fill
    }

    pub fn grid_enabled(&self) -> bool {
        self.grid
    }

    pub fn modified(&self) -> bool {
        self.modified
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn pixel(&self, x: i32, y: i32) -> u32 {
        self.pixels[self.index(x, y)]
    }

    pub fn select_palette_index(&mut self, index: usize) {
        self.cur_color = self.spec.palette[index];
    }

    pub fn toggle_grid(&mut self) {
        self.grid = !self.grid;
    }

    pub fn toggle_fill(&mut self) {
        self.fill = !self.fill;
    }

    pub fn convert_x(&self, local_x: f32, width: f32) -> i32 {
        (local_x / (width / self.spec.canvas_size as f32)) as i32
    }

    pub fn convert_y(&self, local_y: f32, height: f32) -> i32 {
        self.spec.canvas_size - 1 - (local_y / (height / self.spec.canvas_size as f32)) as i32
    }

    pub fn touch_down(
        &mut self,
        local_x: f32,
        local_y: f32,
        width: f32,
        height: f32,
        button: CanvasPointerButton,
    ) -> bool {
        let x = self.convert_x(local_x, width);
        let y = self.convert_y(local_y, height);

        match button {
            CanvasPointerButton::Left if self.fill => {
                if !self.in_bounds(x, y) {
                    return false;
                }
                self.flood_fill(x, y);
                false
            }
            CanvasPointerButton::Left => {
                self.draw_pixel(x, y);
                self.last = Some((x, y));
                true
            }
            CanvasPointerButton::Middle => {
                self.cur_color = self.pixel(x, y);
                false
            }
        }
    }

    pub fn touch_dragged(&mut self, local_x: f32, local_y: f32, width: f32, height: f32) {
        if self.fill {
            return;
        }

        let x = self.convert_x(local_x, width);
        let y = self.convert_y(local_y, height);
        if let Some((last_x, last_y)) = self.last {
            for (px, py) in bresenham_line(last_x, last_y, x, y) {
                self.draw_pixel(px, py);
            }
        }
        self.last = Some((x, y));
    }

    pub fn draw_pixel(&mut self, x: i32, y: i32) {
        if self.in_bounds(x, y) && self.pixel(x, y) != self.cur_color {
            let index = self.index(x, y);
            self.pixels[index] = self.cur_color;
            self.modified = true;
        }
    }

    pub fn import_pixels(&mut self, width: i32, height: i32, pixels: &[u32]) {
        let size = self.spec.canvas_size;
        let mut source_width = width;
        let mut source_height = height;
        let mut source = pixels.to_vec();

        if source_width > size || source_height > size {
            let ratio = source_width.max(source_height) as f32 / size as f32;
            let scaled_width = (source_width as f32 / ratio) as i32;
            let scaled_height = (source_height as f32 / ratio) as i32;
            let mut dest = vec![0; (size * size) as usize];
            let off_x = (size - scaled_width) / 2;
            let off_y = (size - scaled_height) / 2;
            for y in 0..scaled_height {
                for x in 0..scaled_width {
                    let sx = (x as f32 * source_width as f32 / scaled_width as f32) as i32;
                    let sy = (y as f32 * source_height as f32 / scaled_height as f32) as i32;
                    dest[((off_y + y) * size + off_x + x) as usize] =
                        source[(sy * source_width + sx) as usize];
                }
            }
            source = dest;
            source_width = size;
            source_height = size;
        } else if source_width < size || source_height < size {
            let mut dest = vec![self.spec.palette[0]; (size * size) as usize];
            let off_x = (size - source_width) / 2;
            let off_y = (size - source_height) / 2;
            for y in 0..source_height {
                for x in 0..source_width {
                    dest[((off_y + y) * size + off_x + x) as usize] =
                        source[(y * source_width + x) as usize];
                }
            }
            source = dest;
            source_width = size;
            source_height = size;
        }

        let size_x = source_width.min(size);
        let size_y = source_height.min(size);
        for y in 0..size_y {
            for x in 0..size_x {
                let color = source[(y * source_width + x) as usize];
                let closest = self.find_closest(color);
                let index = self.index(x, y);
                self.pixels[index] = closest;
            }
        }

        self.modified = true;
    }

    pub fn find_closest(&self, mut color: u32) -> u32 {
        if rgba_alpha(color) < 255 {
            color = blend_rgba(self.spec.palette[0], color);
        }

        if self.spec.palette.contains(&color) {
            return color;
        }

        let mut nearest = 0usize;
        let mut nearest_dst = f32::MAX;
        for (index, candidate) in self.spec.palette.iter().copied().enumerate() {
            let dst = color_distance_rgba(color, candidate);
            if dst < nearest_dst {
                nearest = index;
                nearest_dst = dst;
            }
        }
        self.spec.palette[nearest]
    }

    pub fn tick(&mut self, delta: f32, canvas_valid: bool) -> Vec<CanvasEditAction> {
        let mut actions = Vec::new();
        if !canvas_valid {
            actions.push(CanvasEditAction::Hide);
        }

        self.time += delta;
        if self.time >= CANVAS_EDIT_REFRESH_TIME {
            if let Some(action) = self.save(canvas_valid) {
                actions.push(action);
            }
            self.time = 0.0;
        }
        actions
    }

    pub fn save(&mut self, canvas_valid: bool) -> Option<CanvasEditAction> {
        if self.modified && canvas_valid {
            let state = self.pack_state();
            self.modified = false;
            Some(CanvasEditAction::Configure(state))
        } else {
            None
        }
    }

    pub fn pack_state(&self) -> CanvasBlockState {
        let bits_per_pixel = self.spec.bits_per_pixel();
        let mut data = vec![0; self.spec.data_len()];
        for (pos, color) in self.pixels.iter().copied().enumerate() {
            let palette_index = self
                .spec
                .palette
                .iter()
                .position(|candidate| *candidate == color)
                .unwrap();
            canvas_set_pixel(
                &mut data,
                self.spec.canvas_size,
                self.spec.palette.len(),
                bits_per_pixel,
                pos as i32,
                palette_index as i32,
            );
        }
        CanvasBlockState::from_data(data)
    }

    pub fn draw_plan(
        &self,
        width: f32,
        height: f32,
        hover_local: Option<(f32, f32)>,
    ) -> CanvasDrawPlan {
        let cell_width = width / self.spec.canvas_size as f32;
        let cell_height = height / self.spec.canvas_size as f32;
        let mut grid_lines = Vec::new();

        if self.grid {
            let minspace: f32 = 10.0;
            let jump_x = (minspace.max(cell_width) / cell_width) as i32;
            let jump_y = (minspace.max(cell_height) / cell_height) as i32;

            for x in (0..=self.spec.canvas_size).step_by(jump_x as usize) {
                let px = cell_width * x as f32;
                grid_lines.push(CanvasGridLine {
                    start: (px, 0.0),
                    end: (px, height),
                });
            }

            for y in (0..=self.spec.canvas_size).step_by(jump_y as usize) {
                let py = cell_height * y as f32;
                grid_lines.push(CanvasGridLine {
                    start: (0.0, py),
                    end: (width, py),
                });
            }
        }

        let hover_cell =
            hover_local.map(|(x, y)| (self.convert_x(x, width), self.convert_y(y, height)));
        CanvasDrawPlan {
            grid: self.grid,
            cell_width,
            cell_height,
            grid_lines,
            hover_cell,
        }
    }

    fn flood_fill(&mut self, x: i32, y: i32) {
        let dst = self.pixel(x, y);
        if self.cur_color == dst {
            return;
        }

        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            if !self.in_bounds(cx, cy) || self.pixel(cx, cy) != dst {
                continue;
            }

            self.draw_pixel(cx, cy);
            stack.extend([(cx + 1, cy), (cx - 1, cy), (cx, cy + 1), (cx, cy - 1)]);
        }
    }

    fn index(&self, x: i32, y: i32) -> usize {
        (y * self.spec.canvas_size + x) as usize
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.spec.canvas_size && y < self.spec.canvas_size
    }
}

fn bresenham_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        points.push((x, y));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }

    points
}

fn rgba_alpha(color: u32) -> u32 {
    color & 0xff
}

fn rgba_components(color: u32) -> (f32, f32, f32, f32) {
    (
        ((color >> 24) & 0xff) as f32 / 255.0,
        ((color >> 16) & 0xff) as f32 / 255.0,
        ((color >> 8) & 0xff) as f32 / 255.0,
        (color & 0xff) as f32 / 255.0,
    )
}

fn blend_rgba(dst: u32, src: u32) -> u32 {
    let (dr, dg, db, _) = rgba_components(dst);
    let (sr, sg, sb, sa) = rgba_components(src);
    let r = sr * sa + dr * (1.0 - sa);
    let g = sg * sa + dg * (1.0 - sa);
    let b = sb * sa + db * (1.0 - sa);
    (((r * 255.0).round() as u32) << 24)
        | (((g * 255.0).round() as u32) << 16)
        | (((b * 255.0).round() as u32) << 8)
        | 0xff
}

fn color_distance_rgba(a: u32, b: u32) -> f32 {
    let (ar, ag, ab, aa) = rgba_components(a);
    let (br, bg, bb, ba) = rgba_components(b);
    ((ar - br).powi(2) + (ag - bg).powi(2) + (ab - bb).powi(2) + (aa - ba).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> CanvasEditSpec {
        CanvasEditSpec::new(4, vec![0x000000ff, 0xffffffff, 0xff0000ff, 0x00ff00ff])
    }

    #[test]
    fn blank_dialog_uses_palette_zero_grid_enabled_and_java_bpp_len() {
        let dialog = CanvasEditDialog::new_blank(spec());

        assert_eq!(dialog.selected_color(), 0x000000ff);
        assert!(dialog.grid_enabled());
        assert!(!dialog.fill_enabled());
        assert_eq!(dialog.pixels(), vec![0x000000ff; 16]);
        assert_eq!(dialog.spec.bits_per_pixel(), 2);
        assert_eq!(dialog.spec.data_len(), 4);
    }

    #[test]
    fn left_click_draws_single_pixel_and_middle_click_samples_color() {
        let mut dialog = CanvasEditDialog::new_blank(spec());
        dialog.select_palette_index(1);

        assert!(dialog.touch_down(10.0, 10.0, 40.0, 40.0, CanvasPointerButton::Left));
        assert_eq!(dialog.pixel(1, 2), 0xffffffff);
        assert!(dialog.modified());

        dialog.select_palette_index(2);
        assert!(!dialog.touch_down(10.0, 10.0, 40.0, 40.0, CanvasPointerButton::Middle));
        assert_eq!(dialog.selected_color(), 0xffffffff);
    }

    #[test]
    fn dragging_draws_bresenham_line_from_last_pixel() {
        let mut dialog = CanvasEditDialog::new_blank(spec());
        dialog.select_palette_index(1);

        dialog.touch_down(0.0, 35.0, 40.0, 40.0, CanvasPointerButton::Left);
        dialog.touch_dragged(30.0, 5.0, 40.0, 40.0);

        assert_eq!(dialog.pixel(0, 0), 0xffffffff);
        assert_eq!(dialog.pixel(1, 1), 0xffffffff);
        assert_eq!(dialog.pixel(2, 2), 0xffffffff);
        assert_eq!(dialog.pixel(3, 3), 0xffffffff);
    }

    #[test]
    fn fill_mode_flood_fills_four_connected_region_only() {
        let mut dialog = CanvasEditDialog::new_blank(spec());
        dialog.select_palette_index(1);
        dialog.draw_pixel(1, 0);
        dialog.draw_pixel(1, 1);
        dialog.draw_pixel(1, 2);
        dialog.draw_pixel(1, 3);

        dialog.select_palette_index(2);
        dialog.toggle_fill();
        dialog.touch_down(0.0, 35.0, 40.0, 40.0, CanvasPointerButton::Left);

        assert_eq!(dialog.pixel(0, 0), 0xff0000ff);
        assert_eq!(dialog.pixel(0, 3), 0xff0000ff);
        assert_eq!(dialog.pixel(1, 0), 0xffffffff);
        assert_eq!(dialog.pixel(2, 0), 0x000000ff);
    }

    #[test]
    fn save_packs_palette_indices_and_clears_modified_only_when_valid() {
        let mut dialog = CanvasEditDialog::new_blank(spec());
        dialog.select_palette_index(2);
        dialog.draw_pixel(2, 1);

        assert_eq!(dialog.save(false), None);
        assert!(dialog.modified());

        let Some(CanvasEditAction::Configure(state)) = dialog.save(true) else {
            panic!("expected configure action");
        };
        assert!(!dialog.modified());
        let bpp = dialog.spec.bits_per_pixel();
        assert_eq!(canvas_get_pixel(&state.data, 4, bpp, 1 * 4 + 2), 2.0);
    }

    #[test]
    fn tick_autosaves_after_refresh_time_and_hides_invalid_canvas() {
        let mut dialog = CanvasEditDialog::new_blank(spec());
        dialog.select_palette_index(1);
        dialog.draw_pixel(0, 0);

        let actions = dialog.tick(CANVAS_EDIT_REFRESH_TIME, true);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], CanvasEditAction::Configure(_)));

        let actions = dialog.tick(CANVAS_EDIT_REFRESH_TIME, false);
        assert_eq!(actions, vec![CanvasEditAction::Hide]);
    }

    #[test]
    fn import_centers_small_sources_and_maps_to_nearest_palette_color() {
        let mut dialog = CanvasEditDialog::new_blank(spec());
        dialog.import_pixels(2, 2, &[0xfefefeff, 0xff1000ff, 0x00f000ff, 0x000000ff]);

        assert_eq!(dialog.pixel(1, 1), 0xffffffff);
        assert_eq!(dialog.pixel(2, 1), 0xff0000ff);
        assert_eq!(dialog.pixel(1, 2), 0x00ff00ff);
        assert_eq!(dialog.pixel(0, 0), 0x000000ff);
    }

    #[test]
    fn draw_plan_reports_grid_lines_and_hover_cell_with_y_flip() {
        let dialog = CanvasEditDialog::new_blank(spec());
        let plan = dialog.draw_plan(40.0, 40.0, Some((15.0, 5.0)));

        assert!(plan.grid);
        assert_eq!(plan.cell_width, 10.0);
        assert_eq!(plan.cell_height, 10.0);
        assert_eq!(plan.hover_cell, Some((1, 3)));
        assert!(!plan.grid_lines.is_empty());
    }
}
