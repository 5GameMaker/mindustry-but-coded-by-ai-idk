#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrailPoint {
    pub x: f32,
    pub y: f32,
    pub width: f32,
}

impl TrailPoint {
    pub const fn new(x: f32, y: f32, width: f32) -> Self {
        Self { x, y, width }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrailSegmentPlan {
    pub index: usize,
    pub total: usize,
    pub start: TrailPoint,
    pub end: TrailPoint,
    pub angle: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrailQuadPlan {
    pub index: usize,
    pub total: usize,
    pub corners: [(f32, f32); 4],
    pub start_width: f32,
    pub end_width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trail {
    pub length: usize,
    pub points: Vec<TrailPoint>,
    pub last_x: f32,
    pub last_y: f32,
    pub last_angle: f32,
    pub counter: f32,
    pub last_width: f32,
}

impl Default for Trail {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Trail {
    pub fn new(length: usize) -> Self {
        Self {
            length,
            points: Vec::with_capacity(length),
            last_x: -1.0,
            last_y: -1.0,
            last_angle: -1.0,
            counter: 0.0,
            last_width: 0.0,
        }
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn width(&self) -> f32 {
        self.last_width
    }

    pub fn clear(&mut self) {
        self.points.clear();
    }

    pub fn size(&self) -> usize {
        self.points.len()
    }

    pub fn points(&self) -> &[TrailPoint] {
        &self.points
    }

    pub fn update(&mut self, x: f32, y: f32) {
        self.update_with_width_and_delta(x, y, 1.0, 1.0);
    }

    pub fn update_with_width(&mut self, x: f32, y: f32, width: f32) {
        self.update_with_width_and_delta(x, y, width, 1.0);
    }

    pub fn update_with_delta(&mut self, x: f32, y: f32, width: f32, delta: f32) {
        self.update_with_width_and_delta(x, y, width, delta);
    }

    pub fn update_with_width_and_delta(&mut self, x: f32, y: f32, width: f32, delta: f32) {
        let count = self.consume_counter(delta);

        if count > 0 {
            let remove = self
                .points
                .len()
                .saturating_add(count.saturating_sub(1))
                .saturating_sub(self.length);

            if remove > 0 {
                self.points.drain(0..remove.min(self.points.len()));
            }

            if count == 1 || self.last_x == -1.0 {
                self.points.push(TrailPoint::new(x, y, width));
            } else {
                for i in 0..count {
                    let factor = (i as f32 + 1.0) / count as f32;
                    self.points.push(TrailPoint::new(
                        lerp(self.last_x, x, factor),
                        lerp(self.last_y, y, factor),
                        lerp(self.last_width, width, factor),
                    ));
                }
            }
        }

        self.last_angle = -angle_rad(x, y, self.last_x, self.last_y);
        self.last_x = x;
        self.last_y = y;
        self.last_width = width;
    }

    pub fn shorten(&mut self) {
        self.shorten_with_delta(1.0);
    }

    pub fn shorten_with_delta(&mut self, delta: f32) {
        let count = self.consume_counter(delta);

        if count > 0 && !self.points.is_empty() {
            let remove = count.min(self.points.len());
            self.points.drain(0..remove);
        }
    }

    pub fn segment_plans(&self) -> Vec<TrailSegmentPlan> {
        let mut plans = Vec::new();
        if self.points.is_empty() {
            return plans;
        }

        for (index, start) in self.points.iter().copied().enumerate() {
            let end = if index + 1 < self.points.len() {
                self.points[index + 1]
            } else {
                TrailPoint::new(self.last_x, self.last_y, self.last_width)
            };

            plans.push(TrailSegmentPlan {
                index,
                total: self.points.len(),
                start,
                end,
                angle: angle_rad(start.x, start.y, end.x, end.y),
            });
        }

        plans
    }

    pub fn quad_plans(&self) -> Vec<TrailQuadPlan> {
        self.segment_plans()
            .into_iter()
            .map(|segment| {
                let dx = segment.end.x - segment.start.x;
                let dy = segment.end.y - segment.start.y;
                let length = (dx * dx + dy * dy).sqrt();
                let (nx, ny) = if length <= f32::EPSILON {
                    (0.0, 0.0)
                } else {
                    (-dy / length, dx / length)
                };
                let half_width = (segment.start.width + segment.end.width) * 0.5;

                TrailQuadPlan {
                    index: segment.index,
                    total: segment.total,
                    corners: [
                        (
                            segment.start.x + nx * half_width,
                            segment.start.y + ny * half_width,
                        ),
                        (
                            segment.start.x - nx * half_width,
                            segment.start.y - ny * half_width,
                        ),
                        (
                            segment.end.x - nx * half_width,
                            segment.end.y - ny * half_width,
                        ),
                        (
                            segment.end.x + nx * half_width,
                            segment.end.y + ny * half_width,
                        ),
                    ],
                    start_width: segment.start.width,
                    end_width: segment.end.width,
                }
            })
            .collect()
    }

    fn consume_counter(&mut self, delta: f32) -> usize {
        let total = (self.counter + delta.max(0.0)).max(0.0);
        let count = total.floor() as usize;
        self.counter = total - count as f32;
        count
    }
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

fn angle_rad(x: f32, y: f32, x2: f32, y2: f32) -> f32 {
    (y2 - y).atan2(x2 - x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trail_defaults_and_copy_preserve_state() {
        let trail = Trail::new(4);
        assert_eq!(trail.length, 4);
        assert_eq!(trail.size(), 0);
        assert_eq!(trail.width(), 0.0);
        assert_eq!(trail.last_x, -1.0);
        assert_eq!(trail.last_y, -1.0);

        let mut updated = trail.copy();
        updated.update_with_width(3.0, 4.0, 2.0);
        let copied = updated.copy();

        assert_eq!(copied.length, 4);
        assert_eq!(copied.points, updated.points);
        assert_eq!(copied.last_x, 3.0);
        assert_eq!(copied.last_y, 4.0);
        assert_eq!(copied.last_width, 2.0);
    }

    #[test]
    fn trail_update_interpolates_points_and_tracks_width() {
        let mut trail = Trail::new(3);
        trail.update_with_width_and_delta(0.0, 0.0, 1.0, 2.0);

        assert_eq!(trail.size(), 1);
        assert_eq!(trail.points[0], TrailPoint::new(0.0, 0.0, 1.0));
        assert_eq!(trail.width(), 1.0);
        assert_eq!(trail.counter, 0.0);

        trail.update_with_width_and_delta(10.0, 0.0, 3.0, 1.0);
        assert_eq!(trail.last_x, 10.0);
        assert_eq!(trail.last_width, 3.0);
        assert!(!trail.points.is_empty());
        assert!(trail.segment_plans().len() >= trail.points.len());
        assert!(!trail.quad_plans().is_empty());
    }

    #[test]
    fn trail_shorten_and_clear_remove_points_without_resetting_last_state() {
        let mut trail = Trail::new(8);
        trail.update_with_width_and_delta(1.0, 2.0, 1.0, 3.0);
        assert!(!trail.points.is_empty());

        trail.shorten_with_delta(2.0);
        assert!(trail.size() <= 1);

        trail.clear();
        assert_eq!(trail.size(), 0);
        assert_eq!(trail.last_x, 1.0);
        assert_eq!(trail.last_y, 2.0);
        assert_eq!(trail.last_width, 1.0);
    }
}
