//! Team registry and palette data mirroring upstream `mindustry.game.Team`.
//!
//! This focuses on deterministic IDs/names/palettes used by saves, packets and
//! logic. World-state helpers such as `core()`/`active()` are intentionally
//! left for the `Teams` runtime migration.

use crate::mindustry::logic::rgba_u32_to_double_bits;

pub const TEAM_COUNT: usize = 256;
pub const BASE_TEAM_COUNT: usize = 6;

pub const TEAM_DERELICT: u8 = 0;
pub const TEAM_SHARDED: u8 = 1;
pub const TEAM_CRUX: u8 = 2;
pub const TEAM_MALIS: u8 = 3;
pub const TEAM_GREEN: u8 = 4;
pub const TEAM_BLUE: u8 = 5;
pub const TEAM_NEOPLASTIC: u8 = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Team {
    pub id: u8,
    pub name: String,
    pub color_rgba: u32,
    pub palette: [u32; 3],
    pub ignore_unit_cap: bool,
    pub emoji: String,
    pub has_palette: bool,
}

impl Team {
    pub fn new(id: u8, name: impl Into<String>, color_rgba: u32) -> Self {
        let mut team = Self {
            id,
            name: name.into(),
            color_rgba,
            palette: [color_rgba; 3],
            ignore_unit_cap: false,
            emoji: String::new(),
            has_palette: false,
        };
        team.set_palette_from_color(color_rgba);
        team
    }

    pub fn with_palette(
        id: u8,
        name: impl Into<String>,
        color_rgba: u32,
        palette: [u32; 3],
    ) -> Self {
        let mut team = Self::new(id, name, color_rgba);
        team.set_palette(palette);
        team.color_rgba = color_rgba;
        team
    }

    pub fn set_palette_from_color(&mut self, color_rgba: u32) {
        self.set_palette([
            color_rgba,
            rgba_mul_rgb(color_rgba, 0.75),
            rgba_mul_rgb(color_rgba, 0.5),
        ]);
        self.has_palette = false;
    }

    pub fn set_palette(&mut self, palette: [u32; 3]) {
        self.color_rgba = palette[0];
        self.palette = palette;
        self.has_palette = true;
    }

    pub fn palettei(&self) -> [i32; 3] {
        [
            self.palette[0] as i32,
            self.palette[1] as i32,
            self.palette[2] as i32,
        ]
    }

    pub fn localized_token(&self) -> String {
        format!("team.{}.name:{}", self.name, self.name)
    }

    pub fn colored_name_with(&self, localized: &str) -> String {
        format!("{}[#{:08x}]{}[]", self.emoji, self.color_rgba, localized)
    }

    pub fn colored_name_token(&self) -> String {
        self.colored_name_with(&self.name)
    }

    pub fn sense_id(&self) -> f64 {
        self.id as f64
    }

    pub fn sense_color(&self) -> f64 {
        rgba_u32_to_double_bits(self.color_rgba)
    }

    pub fn sense_name(&self) -> &str {
        &self.name
    }

    pub fn is_base_team(&self) -> bool {
        (self.id as usize) < BASE_TEAM_COUNT
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamRegistry {
    teams: Vec<Team>,
}

impl TeamRegistry {
    pub fn new() -> Self {
        let mut teams = Vec::with_capacity(TEAM_COUNT);
        teams.push(Team::new(TEAM_DERELICT, "derelict", 0x4d4e58ff));
        teams.push(Team::with_palette(
            TEAM_SHARDED,
            "sharded",
            0xffd37fff,
            [0xffd37fff, 0xeab678ff, 0xd4816bff],
        ));
        teams.push(Team::with_palette(
            TEAM_CRUX,
            "crux",
            0xf25555ff,
            [0xfc8e6cff, 0xf25555ff, 0xa04553ff],
        ));
        teams.push(Team::with_palette(
            TEAM_MALIS,
            "malis",
            0xa27ce5ff,
            [0xc7a4f5ff, 0x896fd6ff, 0x504cbaff],
        ));
        teams.push(Team::new(TEAM_GREEN, "green", 0x54d67dff));
        teams.push(Team::new(TEAM_BLUE, "blue", 0x6c87fdff));
        let mut neoplastic = Team::new(TEAM_NEOPLASTIC, "neoplastic", 0xe05438ff);
        neoplastic.ignore_unit_cap = true;
        teams.push(neoplastic);

        for id in 7..TEAM_COUNT {
            let color = placeholder_team_color(id as u8);
            teams.push(Team::new(id as u8, format!("team#{id}"), color));
        }

        Self { teams }
    }

    pub fn get(&self, id: i32) -> &Team {
        &self.teams[(id as u8) as usize]
    }

    pub fn all(&self) -> &[Team] {
        &self.teams
    }

    pub fn base_teams(&self) -> &[Team] {
        &self.teams[..BASE_TEAM_COUNT]
    }
}

impl Default for TeamRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn vanilla_teams() -> TeamRegistry {
    TeamRegistry::new()
}

fn rgba_mul_rgb(rgba: u32, mul: f32) -> u32 {
    let r = (((rgba >> 24) & 0xff) as f32 * mul).clamp(0.0, 255.0) as u32;
    let g = (((rgba >> 16) & 0xff) as f32 * mul).clamp(0.0, 255.0) as u32;
    let b = (((rgba >> 8) & 0xff) as f32 * mul).clamp(0.0, 255.0) as u32;
    let a = rgba & 0xff;
    (r << 24) | (g << 16) | (b << 8) | a
}

fn placeholder_team_color(id: u8) -> u32 {
    let hash = splitmix32(id as u32 + 0x9e37_79b9);
    let r = 0x60 + ((hash >> 16) & 0x7f);
    let g = 0x60 + ((hash >> 8) & 0x7f);
    let b = 0x60 + (hash & 0x7f);
    (r << 24) | (g << 16) | (b << 8) | 0xff
}

fn splitmix32(mut value: u32) -> u32 {
    value = value.wrapping_add(0x9e37_79b9);
    value = (value ^ (value >> 16)).wrapping_mul(0x85eb_ca6b);
    value = (value ^ (value >> 13)).wrapping_mul(0xc2b2_ae35);
    value ^ (value >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::logic::double_bits_to_rgba;

    #[test]
    fn fixed_team_ids_names_and_palette_match_upstream_constants() {
        let teams = vanilla_teams();
        assert_eq!(teams.all().len(), 256);
        assert_eq!(teams.base_teams().len(), 6);

        let fixed = [
            (TEAM_DERELICT, "derelict", 0x4d4e58ff),
            (TEAM_SHARDED, "sharded", 0xffd37fff),
            (TEAM_CRUX, "crux", 0xf25555ff),
            (TEAM_MALIS, "malis", 0xa27ce5ff),
            (TEAM_GREEN, "green", 0x54d67dff),
            (TEAM_BLUE, "blue", 0x6c87fdff),
            (TEAM_NEOPLASTIC, "neoplastic", 0xe05438ff),
        ];

        for (id, name, color) in fixed {
            let team = teams.get(id as i32);
            assert_eq!(team.id, id);
            assert_eq!(team.name, name);
            assert_eq!(team.color_rgba, color);
        }

        assert_eq!(
            teams.get(TEAM_SHARDED as i32).palette,
            [0xffd37fff, 0xeab678ff, 0xd4816bff]
        );
        assert_eq!(
            teams.get(TEAM_CRUX as i32).palette,
            [0xfc8e6cff, 0xf25555ff, 0xa04553ff]
        );
        assert!(teams.get(TEAM_NEOPLASTIC as i32).ignore_unit_cap);
    }

    #[test]
    fn get_uses_java_byte_mask_for_team_ids() {
        let teams = vanilla_teams();
        assert_eq!(teams.get(-1).id, 255);
        assert_eq!(teams.get(256).id, 0);
        assert_eq!(teams.get(257).id, 1);
        assert_eq!(teams.get(300).id, 44);
        assert_eq!(teams.get(44).name, "team#44");
    }

    #[test]
    fn set_palette_from_color_matches_java_default_palette_shape() {
        let team = Team::new(9, "test", 0x80_40_20_ff);
        assert!(!team.has_palette);
        assert_eq!(team.palette, [0x804020ff, 0x603018ff, 0x402010ff]);
        assert_eq!(
            team.palettei(),
            [0x804020ff_u32 as i32, 0x603018ff, 0x402010ff]
        );
    }

    #[test]
    fn with_palette_marks_custom_palette_and_restores_display_color_like_java_constructor() {
        let team = Team::with_palette(
            42,
            "custom",
            0x11223344,
            [0xaabbccdd, 0x55667788, 0x01020304],
        );

        assert!(team.has_palette);
        assert_eq!(team.color_rgba, 0x11223344);
        assert_eq!(team.palette, [0xaabbccdd, 0x55667788, 0x01020304]);
        assert_eq!(
            team.palettei(),
            [
                0xaabbccdd_u32 as i32,
                0x55667788_u32 as i32,
                0x01020304_u32 as i32,
            ]
        );
    }

    #[test]
    fn colored_name_token_matches_java_color_markup_shape() {
        let mut team = Team::new(8, "alpha", 0x0a1b2c3d);
        assert_eq!(team.colored_name_token(), "[#0a1b2c3d]alpha[]");
        assert_eq!(
            team.colored_name_with("Alpha Team"),
            "[#0a1b2c3d]Alpha Team[]"
        );

        team.emoji = "⚑".into();
        assert_eq!(team.colored_name_token(), "⚑[#0a1b2c3d]alpha[]");
    }

    #[test]
    fn sense_helpers_match_logic_team_contract() {
        let teams = vanilla_teams();
        let crux = teams.get(TEAM_CRUX as i32);
        assert_eq!(crux.sense_id(), 2.0);
        assert_eq!(double_bits_to_rgba(crux.sense_color()), 0xf25555ff);
        assert_eq!(crux.sense_name(), "crux");
        assert_eq!(crux.localized_token(), "team.crux.name:crux");
    }
}
