use super::AsyncProcess;

pub const AVOIDANCE_LAYER_GROUND: i32 = 0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvoidanceUnit {
    pub id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
    pub hit_size: f32,
    pub collision_layer: i32,
    pub team_is_ai: bool,
    pub team_rts_ai: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvoidanceRequest {
    pub tile_x: i32,
    pub tile_y: i32,
    pub radius: f32,
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AvoidanceProcess {
    pub width: usize,
    pub height: usize,
    buffer1: Option<Vec<i32>>,
    buffer2: Option<Vec<i32>>,
    pub swap: bool,
    pub requests: Vec<AvoidanceRequest>,
    avoidance: Option<Vec<i32>>,
    pub modified: bool,
    pub active: bool,
}

impl AvoidanceProcess {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer1: None,
            buffer2: None,
            swap: false,
            requests: Vec::new(),
            avoidance: None,
            modified: false,
            active: false,
        }
    }

    pub fn init_world(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    pub fn get_avoidance(&mut self) -> Option<&[i32]> {
        if !self.active {
            let len = self.width * self.height;
            self.buffer1 = Some(vec![0; len]);
            self.buffer2 = Some(vec![0; len]);
            self.active = true;
        }
        self.avoidance.as_deref()
    }

    pub fn begin_with_units(
        &mut self,
        units: &[AvoidanceUnit],
        unit_collision_radius_scale: f32,
        tile_size: f32,
    ) {
        if !self.active {
            return;
        }

        self.requests.clear();
        self.avoidance = if !self.swap {
            self.buffer1.clone()
        } else {
            self.buffer2.clone()
        };

        for unit in units {
            if unit.team_is_ai
                && !unit.team_rts_ai
                && unit.collision_layer == AVOIDANCE_LAYER_GROUND
            {
                let scaling = 2.0;
                self.requests.push(AvoidanceRequest {
                    tile_x: unit.tile_x,
                    tile_y: unit.tile_y,
                    radius: unit.hit_size * unit_collision_radius_scale / tile_size * scaling,
                    id: unit.id,
                });
            }
        }
    }

    pub fn process_requests(&mut self) {
        let target = if self.swap {
            &mut self.buffer1
        } else {
            &mut self.buffer2
        };
        self.swap = !self.swap;

        let Some(buffer) = target.as_mut() else {
            return;
        };

        if self.modified {
            buffer.fill(0);
        }

        self.modified = !self.requests.is_empty();

        for request in &self.requests {
            let radius2 = request.radius * request.radius;
            let r = request.radius.ceil().max(1.0) as i32;
            for dx in -r..=r {
                for dy in -r..=r {
                    let x = dx + request.tile_x;
                    let y = dy + request.tile_y;
                    if x >= 0
                        && y >= 0
                        && (x as usize) < self.width
                        && (y as usize) < self.height
                        && (dx * dx + dy * dy) as f32 <= radius2
                    {
                        let index = x as usize + y as usize * self.width;
                        buffer[index] = buffer[index].max(i32::MAX - request.id);
                    }
                }
            }
        }
    }

    pub fn active_buffer(&self) -> Option<&[i32]> {
        self.avoidance.as_deref()
    }

    pub fn processing_buffer(&self) -> Option<&[i32]> {
        if self.swap {
            self.buffer2.as_deref()
        } else {
            self.buffer1.as_deref()
        }
    }
}

impl AsyncProcess for AvoidanceProcess {
    fn reset(&mut self) {
        self.buffer1 = None;
        self.buffer2 = None;
        self.avoidance = None;
        self.swap = false;
        self.modified = false;
        self.active = false;
        self.requests.clear();
    }

    fn process(&mut self) {
        self.process_requests();
    }

    fn should_process(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avoidance_get_avoidance_lazily_initializes_buffers_but_returns_previous_field() {
        let mut process = AvoidanceProcess::new(4, 3);

        assert_eq!(process.get_avoidance(), None);
        assert!(process.active);
        assert!(process.should_process());
        assert_eq!(process.buffer1.as_ref().unwrap().len(), 12);
        assert_eq!(process.buffer2.as_ref().unwrap().len(), 12);
    }

    #[test]
    fn avoidance_begin_collects_only_ai_ground_non_rts_units() {
        let mut process = AvoidanceProcess::new(8, 8);
        process.get_avoidance();

        process.begin_with_units(
            &[
                AvoidanceUnit {
                    id: 10,
                    tile_x: 2,
                    tile_y: 3,
                    hit_size: 8.0,
                    collision_layer: AVOIDANCE_LAYER_GROUND,
                    team_is_ai: true,
                    team_rts_ai: false,
                },
                AvoidanceUnit {
                    id: 11,
                    tile_x: 5,
                    tile_y: 5,
                    hit_size: 8.0,
                    collision_layer: 2,
                    team_is_ai: true,
                    team_rts_ai: false,
                },
                AvoidanceUnit {
                    id: 12,
                    tile_x: 6,
                    tile_y: 6,
                    hit_size: 8.0,
                    collision_layer: AVOIDANCE_LAYER_GROUND,
                    team_is_ai: true,
                    team_rts_ai: true,
                },
            ],
            0.6,
            8.0,
        );

        assert_eq!(
            process.requests,
            vec![AvoidanceRequest {
                tile_x: 2,
                tile_y: 3,
                radius: 1.2,
                id: 10,
            }]
        );
        assert_eq!(process.active_buffer().unwrap(), vec![0; 64].as_slice());
    }

    #[test]
    fn avoidance_process_draws_radius_into_inactive_buffer_and_swaps_each_tick() {
        let mut process = AvoidanceProcess::new(5, 5);
        process.get_avoidance();
        process.requests.push(AvoidanceRequest {
            tile_x: 2,
            tile_y: 2,
            radius: 1.1,
            id: 7,
        });

        process.process_requests();
        assert!(process.swap);
        assert!(process.modified);

        let processed = process.processing_buffer().unwrap();
        let value = i32::MAX - 7;
        assert_eq!(processed[2 + 2 * 5], value);
        assert_eq!(processed[1 + 2 * 5], value);
        assert_eq!(processed[3 + 2 * 5], value);
        assert_eq!(processed[2 + 1 * 5], value);
        assert_eq!(processed[2 + 3 * 5], value);
        assert_eq!(processed[0], 0);

        process.requests.clear();
        process.process_requests();
        assert!(!process.swap);
        assert!(!process.modified);
    }

    #[test]
    fn avoidance_reset_clears_all_runtime_buffers_and_state() {
        let mut process = AvoidanceProcess::new(2, 2);
        process.get_avoidance();
        process.requests.push(AvoidanceRequest {
            tile_x: 0,
            tile_y: 0,
            radius: 1.0,
            id: 1,
        });
        process.modified = true;

        process.reset();

        assert!(!process.active);
        assert!(!process.modified);
        assert!(!process.swap);
        assert!(process.requests.is_empty());
        assert!(process.buffer1.is_none());
        assert!(process.buffer2.is_none());
        assert!(process.active_buffer().is_none());
    }
}
