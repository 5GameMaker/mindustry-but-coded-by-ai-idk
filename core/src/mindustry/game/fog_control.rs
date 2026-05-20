#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FogEvent(pub u64);

impl FogEvent {
    pub fn get(x: i32, y: i32, radius: i32, team: i32) -> Self {
        Self(
            (x as u16 as u64)
                | ((y as u16 as u64) << 16)
                | ((radius as u16 as u64) << 32)
                | ((team as u8 as u64) << 48),
        )
    }

    pub fn x(self) -> i32 {
        (self.0 & 0xffff) as i32
    }

    pub fn y(self) -> i32 {
        ((self.0 >> 16) & 0xffff) as i32
    }

    pub fn radius(self) -> i32 {
        ((self.0 >> 32) & 0xffff) as i32
    }

    pub fn team(self) -> u8 {
        ((self.0 >> 48) & 0xff) as u8
    }
}

impl From<FogEvent> for u64 {
    fn from(value: FogEvent) -> Self {
        value.0
    }
}

impl From<u64> for FogEvent {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FogBits {
    len: usize,
    words: Vec<u64>,
}

impl FogBits {
    pub fn new(len: usize) -> Self {
        Self {
            len,
            words: vec![0; len.div_ceil(64)],
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.words.fill(0);
    }

    pub fn get(&self, index: usize) -> bool {
        index < self.len && (self.words[index / 64] & (1u64 << (index % 64))) != 0
    }

    pub fn set(&mut self, index: usize) {
        if index < self.len {
            self.words[index / 64] |= 1u64 << (index % 64);
        }
    }

    pub fn set_range(&mut self, start: usize, end: usize) {
        for index in start..end.min(self.len) {
            self.set(index);
        }
    }

    pub fn count_ones(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FogData {
    pub read: FogBits,
    pub write: FogBits,
    pub static_data: FogBits,
    pub last_dynamic_ms: u64,
    pub dynamic_updated: bool,
}

impl FogData {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width.saturating_mul(height);
        Self {
            read: FogBits::new(len),
            write: FogBits::new(len),
            static_data: FogBits::new(len),
            last_dynamic_ms: 0,
            dynamic_updated: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FogControl {
    width: usize,
    height: usize,
    fog: Vec<Option<FogData>>,
    static_events: Vec<FogEvent>,
    dynamic_events: Vec<FogEvent>,
}

impl FogControl {
    pub const TEAM_COUNT: usize = 256;
    pub const DYNAMIC_UPDATE_INTERVAL_MS: u64 = 1000 / 25;

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            fog: vec![None; Self::TEAM_COUNT],
            static_events: Vec::new(),
            dynamic_events: Vec::new(),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn reset_world(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.reset_fog();
    }

    pub fn reset_fog(&mut self) {
        self.fog.fill_with(|| None);
        self.static_events.clear();
        self.dynamic_events.clear();
    }

    pub fn data(&self, team: u8) -> Option<&FogData> {
        self.fog.get(team as usize).and_then(Option::as_ref)
    }

    pub fn data_mut(&mut self, team: u8) -> Option<&mut FogData> {
        self.fog.get_mut(team as usize).and_then(Option::as_mut)
    }

    pub fn ensure_data(&mut self, team: u8) -> &mut FogData {
        self.fog[team as usize].get_or_insert_with(|| FogData::new(self.width, self.height))
    }

    pub fn get_discovered(&self, team: u8) -> Option<&FogBits> {
        self.data(team).map(|data| &data.static_data)
    }

    pub fn is_discovered(
        &self,
        fog_enabled: bool,
        static_fog: bool,
        team: Option<u8>,
        team_is_ai: bool,
        x: i32,
        y: i32,
    ) -> bool {
        if !static_fog || !fog_enabled || team.is_none() || team_is_ai {
            return true;
        }

        let Some(data) = self.get_discovered(team.unwrap()) else {
            return false;
        };
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        data.get(x as usize + y as usize * self.width)
    }

    pub fn is_visible_tile(
        &self,
        fog_enabled: bool,
        team: Option<u8>,
        team_is_ai: bool,
        x: i32,
        y: i32,
    ) -> bool {
        if !fog_enabled || team.is_none() || team_is_ai {
            return true;
        }

        let Some(data) = self.data(team.unwrap()) else {
            return false;
        };
        if self.width == 0 || self.height == 0 {
            return false;
        }
        let x = x.clamp(0, self.width as i32 - 1) as usize;
        let y = y.clamp(0, self.height as i32 - 1) as usize;
        data.read.get(x + y * self.width)
    }

    pub fn push_static_event(&mut self, event: FogEvent, static_fog: bool) {
        if static_fog {
            self.static_events.push(event);
        }
    }

    pub fn push_dynamic_event(&mut self, event: FogEvent) {
        self.dynamic_events.push(event);
    }

    pub fn mark_dynamic_updated(&mut self, team: u8) {
        self.ensure_data(team).dynamic_updated = true;
    }

    pub fn force_update(&mut self, event: FogEvent, fog_enabled: bool, static_fog: bool) {
        if fog_enabled && self.data(event.team()).is_some() {
            self.mark_dynamic_updated(event.team());
            self.push_static_event(event, static_fog);
        }
    }

    pub fn update_static(&mut self) {
        let events = std::mem::take(&mut self.static_events);
        for event in events {
            let team = event.team();
            let width = self.width;
            let height = self.height;
            let data = self.ensure_data(team);
            circle(
                &mut data.static_data,
                width,
                height,
                event.x(),
                event.y(),
                event.radius(),
            );
        }
    }

    pub fn update_dynamic(&mut self) {
        let mut cleared = [false; Self::TEAM_COUNT];
        let events = std::mem::take(&mut self.dynamic_events);

        for event in events {
            let radius = event.radius();
            if radius <= 0 {
                continue;
            }

            let team = event.team() as usize;
            let width = self.width;
            let height = self.height;
            let data = self.ensure_data(event.team());

            if !cleared[team] {
                cleared[team] = true;
                data.write.clear();
            }

            circle(
                &mut data.write,
                width,
                height,
                event.x(),
                event.y(),
                radius + 1,
            );
        }

        for (team, was_cleared) in cleared.into_iter().enumerate() {
            if was_cleared {
                let data = self.fog[team]
                    .as_mut()
                    .expect("cleared team must have fog data");
                std::mem::swap(&mut data.read, &mut data.write);
            }
        }
    }

    pub fn should_write(&self, fog_enabled: bool, static_fog: bool) -> bool {
        fog_enabled && static_fog && self.fog.iter().any(Option::is_some)
    }

    pub fn write_static_fog_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let used = self.fog.iter().filter(|entry| entry.is_some()).count() as u8;
        out.push(used);
        out.extend_from_slice(&(self.width as u16).to_be_bytes());
        out.extend_from_slice(&(self.height as u16).to_be_bytes());

        let size = self.width.saturating_mul(self.height);
        for (team, data) in self.fog.iter().enumerate() {
            let Some(data) = data else {
                continue;
            };
            out.push(team as u8);

            let mut pos = 0usize;
            while pos < size {
                let mut consecutives = 0usize;
                let current = data.static_data.get(pos);
                while consecutives < 127 && pos < size {
                    if current != data.static_data.get(pos) {
                        break;
                    }
                    consecutives += 1;
                    pos += 1;
                }

                let mask = if current { 0b1000_0000 } else { 0 };
                out.push(mask | consecutives as u8);
            }
        }
        out
    }

    pub fn read_static_fog_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() < 5 {
            return Err("static fog chunk is too short".into());
        }

        let teams = bytes[0] as usize;
        let width = u16::from_be_bytes([bytes[1], bytes[2]]) as usize;
        let height = u16::from_be_bytes([bytes[3], bytes[4]]) as usize;
        let len = width.saturating_mul(height);

        self.width = width;
        self.height = height;
        self.fog = vec![None; Self::TEAM_COUNT];
        self.static_events.clear();
        self.dynamic_events.clear();

        let mut cursor = 5usize;
        for _ in 0..teams {
            let Some(&team) = bytes.get(cursor) else {
                return Err("static fog chunk ended before team id".into());
            };
            cursor += 1;

            let mut data = FogData::new(width, height);
            let mut pos = 0usize;
            while pos < len {
                let Some(&byte) = bytes.get(cursor) else {
                    return Err("static fog chunk ended inside run-length data".into());
                };
                cursor += 1;

                let set = (byte & 0b1000_0000) != 0;
                let consecutives = (byte & 0b0111_1111) as usize;
                if consecutives == 0 {
                    return Err("static fog chunk contains a zero-length run".into());
                }
                if pos + consecutives > len {
                    return Err("static fog run exceeds world size".into());
                }
                if set {
                    data.static_data.set_range(pos, pos + consecutives);
                }
                pos += consecutives;
            }

            self.fog[team as usize] = Some(data);
        }

        Ok(())
    }
}

pub fn circle(bits: &mut FogBits, width: usize, height: usize, x: i32, y: i32, radius: i32) {
    let mut f = 1 - radius;
    let mut dd_fx = 1;
    let mut dd_fy = -2 * radius;
    let mut px = 0;
    let mut py = radius;

    hline(bits, width, height, x, x, y + radius);
    hline(bits, width, height, x, x, y - radius);
    hline(bits, width, height, x - radius, x + radius, y);

    while px < py {
        if f >= 0 {
            py -= 1;
            dd_fy += 2;
            f += dd_fy;
        }

        px += 1;
        dd_fx += 2;
        f += dd_fx;

        hline(bits, width, height, x - px, x + px, y + py);
        hline(bits, width, height, x - px, x + px, y - py);
        hline(bits, width, height, x - py, x + py, y + px);
        hline(bits, width, height, x - py, x + py, y - px);
    }
}

pub fn hline(bits: &mut FogBits, width: usize, height: usize, mut x1: i32, mut x2: i32, y: i32) {
    if y < 0 || y >= height as i32 {
        return;
    }

    if x1 > x2 {
        std::mem::swap(&mut x1, &mut x2);
    }

    if x1 >= width as i32 || x2 < 0 {
        return;
    }

    x1 = x1.max(0);
    x2 = x2.min(width as i32 - 1);
    let offset = y as usize * width;
    bits.set_range(offset + x1 as usize, offset + x2 as usize + 1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_event_struct_packs_generated_field_order() {
        let event = FogEvent::get(0x1234, 0x5678, 0x9abc, 0xde);
        assert_eq!(event.x(), 0x1234);
        assert_eq!(event.y(), 0x5678);
        assert_eq!(event.radius(), 0x9abc);
        assert_eq!(event.team(), 0xde);
        assert_eq!(u64::from(event), 0x00de_9abc_5678_1234);
    }

    #[test]
    fn circle_and_hline_clip_to_world_bounds_like_java() {
        let mut bits = FogBits::new(25);
        circle(&mut bits, 5, 5, 2, 2, 1);

        let set: Vec<_> = (0..25).filter(|&index| bits.get(index)).collect();
        assert_eq!(set, vec![7, 11, 12, 13, 17]);

        hline(&mut bits, 5, 5, -3, 9, 0);
        for x in 0..5 {
            assert!(bits.get(x));
        }
    }

    #[test]
    fn discovery_and_visibility_follow_fog_rule_shortcuts() {
        let mut fog = FogControl::new(4, 4);
        assert!(fog.is_discovered(false, true, Some(1), false, 1, 1));
        assert!(fog.is_discovered(true, true, None, false, 1, 1));
        assert!(!fog.is_discovered(true, true, Some(1), false, 1, 1));

        fog.push_static_event(FogEvent::get(1, 1, 1, 1), true);
        fog.update_static();
        assert!(fog.is_discovered(true, true, Some(1), false, 1, 1));
        assert!(!fog.is_discovered(true, true, Some(1), false, -1, 1));

        assert!(!fog.is_visible_tile(true, Some(1), false, 1, 1));
        fog.push_dynamic_event(FogEvent::get(1, 1, 1, 1));
        fog.update_dynamic();
        assert!(fog.is_visible_tile(true, Some(1), false, 1, 1));
        assert!(fog.is_visible_tile(false, Some(1), false, 99, 99));
    }

    #[test]
    fn dynamic_update_clears_once_per_team_and_swaps_buffers() {
        let mut fog = FogControl::new(6, 6);
        fog.push_dynamic_event(FogEvent::get(1, 1, 1, 2));
        fog.update_dynamic();
        let first_count = fog.data(2).unwrap().read.count_ones();
        assert!(first_count > 0);

        fog.push_dynamic_event(FogEvent::get(4, 4, 1, 2));
        fog.update_dynamic();
        let data = fog.data(2).unwrap();
        assert!(data.read.get(4 + 4 * 6));
        assert!(!data.read.get(1 + 1 * 6));
        assert_eq!(data.write.count_ones(), first_count);
    }

    #[test]
    fn static_fog_chunk_uses_java_rle_shape() {
        let mut fog = FogControl::new(4, 1);
        {
            let data = fog.ensure_data(3);
            data.static_data.set(0);
            data.static_data.set(1);
        }

        let bytes = fog.write_static_fog_bytes();
        assert_eq!(bytes, vec![1, 0, 4, 0, 1, 3, 0x82, 0x02]);

        let mut decoded = FogControl::new(0, 0);
        decoded.read_static_fog_bytes(&bytes).unwrap();
        assert_eq!(decoded.width(), 4);
        assert_eq!(decoded.height(), 1);
        assert!(decoded.is_discovered(true, true, Some(3), false, 0, 0));
        assert!(decoded.is_discovered(true, true, Some(3), false, 1, 0));
        assert!(!decoded.is_discovered(true, true, Some(3), false, 2, 0));
    }

    #[test]
    fn should_write_requires_fog_static_fog_and_allocated_team_data() {
        let mut fog = FogControl::new(2, 2);
        assert!(!fog.should_write(true, true));
        fog.ensure_data(1);
        assert!(fog.should_write(true, true));
        assert!(!fog.should_write(false, true));
        assert!(!fog.should_write(true, false));
    }
}
