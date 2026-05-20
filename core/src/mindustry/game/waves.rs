//! Wave spawn tables and helpers mirroring upstream `mindustry.game.Waves`.

use crate::mindustry::game::SpawnGroup;
use crate::mindustry::r#type::ItemStack;

pub const WAVE_VERSION: i32 = 7;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Waves {
    spawns: Option<Vec<SpawnGroup>>,
}

impl Waves {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&mut self) -> &[SpawnGroup] {
        if self.spawns.is_none() {
            self.spawns = Some(default_spawn_groups());
        }
        self.spawns.as_deref().unwrap_or(&[])
    }

    pub fn take(self) -> Vec<SpawnGroup> {
        self.spawns.unwrap_or_else(default_spawn_groups)
    }
}

pub fn default_spawn_groups() -> Vec<SpawnGroup> {
    vec![
        group("dagger", |g| {
            g.end = 10;
            g.unit_scaling = 2.0;
            g.max = 30;
        }),
        group("crawler", |g| {
            g.begin = 4;
            g.end = 13;
            g.unit_amount = 2;
            g.unit_scaling = 1.5;
        }),
        group("flare", |g| {
            g.begin = 12;
            g.end = 16;
            g.unit_scaling = 1.0;
        }),
        group("dagger", |g| {
            g.begin = 11;
            g.unit_scaling = 1.7;
            g.spacing = 2;
            g.max = 4;
            g.shield_scaling = 25.0;
        }),
        group("pulsar", |g| {
            g.begin = 13;
            g.spacing = 3;
            g.unit_scaling = 0.5;
            g.max = 25;
        }),
        group("mace", |g| {
            g.begin = 7;
            g.spacing = 3;
            g.unit_scaling = 2.0;
            g.end = 30;
        }),
        group("dagger", |g| {
            g.begin = 12;
            g.unit_scaling = 1.0;
            g.unit_amount = 4;
            g.spacing = 2;
            g.shield_scaling = 20.0;
            g.max = 14;
        }),
        group("mace", |g| {
            g.begin = 28;
            g.spacing = 3;
            g.unit_scaling = 1.0;
            g.end = 40;
            g.shield_scaling = 20.0;
        }),
        group("spiroct", |g| {
            g.begin = 45;
            g.spacing = 3;
            g.unit_scaling = 1.0;
            g.max = 10;
            g.shield_scaling = 30.0;
            g.shields = 100.0;
            g.effect = Some("overdrive".into());
        }),
        group("pulsar", |g| {
            g.begin = 120;
            g.spacing = 2;
            g.unit_scaling = 3.0;
            g.unit_amount = 5;
            g.effect = Some("overdrive".into());
        }),
        group("flare", |g| {
            g.begin = 16;
            g.unit_scaling = 1.0;
            g.spacing = 2;
            g.shield_scaling = 20.0;
            g.max = 20;
        }),
        group("quasar", |g| {
            g.begin = 82;
            g.spacing = 3;
            g.unit_amount = 4;
            g.unit_scaling = 3.0;
            g.shield_scaling = 30.0;
            g.effect = Some("overdrive".into());
        }),
        group("pulsar", |g| {
            g.begin = 41;
            g.spacing = 5;
            g.unit_amount = 1;
            g.unit_scaling = 3.0;
            g.shields = 640.0;
            g.max = 25;
        }),
        group("fortress", |g| {
            g.begin = 40;
            g.spacing = 5;
            g.unit_amount = 2;
            g.unit_scaling = 2.0;
            g.max = 20;
            g.shield_scaling = 30.0;
        }),
        group("nova", |g| {
            g.begin = 35;
            g.spacing = 3;
            g.unit_amount = 4;
            g.effect = Some("overdrive".into());
            g.items = Some(ItemStack::new("blast-compound", 60));
            g.end = 60;
        }),
        group("dagger", |g| {
            g.begin = 42;
            g.spacing = 3;
            g.unit_amount = 4;
            g.effect = Some("overdrive".into());
            g.items = Some(ItemStack::new("pyratite", 100));
            g.end = 130;
            g.max = 30;
        }),
        group("horizon", |g| {
            g.begin = 40;
            g.unit_amount = 2;
            g.spacing = 2;
            g.unit_scaling = 2.0;
            g.shield_scaling = 20.0;
        }),
        group("flare", |g| {
            g.begin = 50;
            g.unit_amount = 4;
            g.unit_scaling = 3.0;
            g.spacing = 5;
            g.shields = 100.0;
            g.shield_scaling = 10.0;
            g.effect = Some("overdrive".into());
            g.max = 20;
        }),
        group("zenith", |g| {
            g.begin = 50;
            g.unit_amount = 2;
            g.unit_scaling = 3.0;
            g.spacing = 5;
            g.max = 16;
            g.shield_scaling = 30.0;
        }),
        group("nova", |g| {
            g.begin = 53;
            g.unit_amount = 2;
            g.unit_scaling = 3.0;
            g.spacing = 4;
            g.shield_scaling = 30.0;
        }),
        group("atrax", |g| {
            g.begin = 31;
            g.unit_amount = 4;
            g.unit_scaling = 1.0;
            g.spacing = 3;
            g.shield_scaling = 10.0;
        }),
        group("scepter", |g| {
            g.begin = 41;
            g.unit_amount = 1;
            g.unit_scaling = 1.0;
            g.spacing = 30;
            g.shield_scaling = 30.0;
        }),
        group("reign", |g| {
            g.begin = 81;
            g.unit_amount = 1;
            g.unit_scaling = 1.0;
            g.spacing = 40;
            g.shield_scaling = 30.0;
        }),
        group("antumbra", |g| {
            g.begin = 120;
            g.unit_amount = 1;
            g.unit_scaling = 1.0;
            g.spacing = 40;
            g.shield_scaling = 30.0;
        }),
        group("vela", |g| {
            g.begin = 100;
            g.unit_amount = 1;
            g.unit_scaling = 1.0;
            g.spacing = 30;
            g.shield_scaling = 30.0;
        }),
        group("corvus", |g| {
            g.begin = 145;
            g.unit_amount = 1;
            g.unit_scaling = 1.0;
            g.spacing = 35;
            g.shield_scaling = 30.0;
            g.shields = 100.0;
        }),
        group("horizon", |g| {
            g.begin = 90;
            g.unit_amount = 2;
            g.unit_scaling = 3.0;
            g.spacing = 4;
            g.shields = 40.0;
            g.shield_scaling = 30.0;
        }),
        group("toxopid", |g| {
            g.begin = 210;
            g.unit_amount = 1;
            g.unit_scaling = 1.0;
            g.spacing = 35;
            g.shields = 1000.0;
            g.shield_scaling = 35.0;
        }),
    ]
}

pub fn generate(difficulty: f32) -> Vec<SpawnGroup> {
    let scaled = difficulty.powf(1.12);
    generate_with_seed(scaled, 0, false, false, false)
}

pub fn generate_with_seed(
    difficulty: f32,
    seed: u64,
    attack: bool,
    air_only: bool,
    naval: bool,
) -> Vec<SpawnGroup> {
    let mut rng = WaveRand::new(seed);
    let mut groups = generated_wave_skeleton(difficulty, &mut rng, attack, air_only, naval);
    let shift = ((difficulty * 14.0 - 5.0) as i32).max(0);
    for group in &mut groups {
        group.begin -= shift;
        group.end = group.end.saturating_sub(shift);
    }
    groups
}

fn generated_wave_skeleton(
    difficulty: f32,
    rng: &mut WaveRand,
    attack: bool,
    air_only: bool,
    naval: bool,
) -> Vec<SpawnGroup> {
    let mut species = vec![
        vec!["dagger", "mace", "fortress", "scepter", "reign"],
        vec!["nova", "pulsar", "quasar", "vela", "corvus"],
        vec!["crawler", "atrax", "spiroct", "arkyid", "toxopid"],
        vec!["risso", "minke", "bryde", "sei", "omura"],
        vec!["retusa", "oxynoe", "cyerce", "aegires", "navanax"],
        vec![
            "flare",
            "horizon",
            "zenith",
            if rng.chance(0.5) { "quad" } else { "antumbra" },
            if rng.chance(0.1) { "quad" } else { "eclipse" },
        ],
    ];

    if air_only {
        species.retain(|row| matches!(row[0], "flare"));
    }
    if naval {
        species.retain(|row| is_air(row[0]) || is_naval(row[0]));
    } else {
        species.retain(|row| !is_naval(row[0]));
    }

    let cap = 150;
    let shield_start = 30.0;
    let shields_per_wave = 20.0 + difficulty * 30.0;
    let scaling = [1.0_f32, 2.0, 3.0, 4.0, 5.0];
    let mut out = Vec::new();

    create_progression(
        0,
        cap,
        difficulty,
        shields_per_wave,
        shield_start,
        &species,
        &scaling,
        rng,
        &mut out,
    );

    let mut step = 5 + rng.random_i32(0, 5);
    while step <= cap {
        create_progression(
            step,
            cap,
            difficulty,
            shields_per_wave,
            shield_start,
            &species,
            &scaling,
            rng,
            &mut out,
        );
        step += (rng.random_range_i32(15, 30) as f32 * lerp(1.0, 0.5, difficulty)) as i32;
    }

    let boss_wave = (rng.random_range_i32(50, 70) as f32 * lerp(1.0, 0.5, difficulty)) as i32;
    let boss_spacing = (rng.random_range_i32(25, 40) as f32 * lerp(1.0, 0.5, difficulty)) as i32;
    let boss_tier = if difficulty < 0.6 { 3 } else { 4 };

    for offset in [0, rng.random_range_i32(3, 5) * boss_spacing] {
        let unit = random_species(&species, rng)[boss_tier].to_string();
        out.push(group(unit, |g| {
            g.unit_amount = 1;
            g.begin = boss_wave + offset;
            g.spacing = boss_spacing;
            g.end = SpawnGroup::NEVER;
            g.max = 16;
            g.unit_scaling = boss_spacing as f32;
            g.shield_scaling = shields_per_wave;
            g.effect = Some("boss".into());
        }));
    }

    let final_boss_start = 120 + rng.random_i32(0, 30);
    for offset in [0, 15] {
        let unit = random_species(&species, rng)[boss_tier].to_string();
        out.push(group(unit, |g| {
            g.unit_amount = 1;
            g.begin = final_boss_start + offset;
            g.spacing = boss_spacing / 2;
            g.end = SpawnGroup::NEVER;
            g.unit_scaling = boss_spacing as f32;
            g.shields = 500.0;
            g.shield_scaling = shields_per_wave * 4.0;
            g.effect = Some("boss".into());
        }));
    }

    if attack && difficulty >= 0.5 {
        let amount = rng.random_range_i32(1, 3 + (difficulty * 2.0) as i32);
        for _ in 0..amount {
            let wave = rng.random_range_i32(3, 20);
            out.push(group("mega", |g| {
                g.unit_amount = 1;
                g.begin = wave;
                g.end = wave;
                g.max = 16;
            }));
        }
    }

    out
}

#[allow(clippy::too_many_arguments)]
fn create_progression(
    start: i32,
    cap: i32,
    difficulty: f32,
    shields_per_wave: f32,
    shield_start: f32,
    species: &[Vec<&str>],
    scaling: &[f32; 5],
    rng: &mut WaveRand,
    out: &mut Vec<SpawnGroup>,
) {
    if species.is_empty() {
        return;
    }
    let mut cur_species = random_species(species, rng).clone();
    let mut cur_tier = 0usize;
    let mut i = start;
    while i < cap {
        let f = i;
        let next =
            rng.random_range_i32(8, 16) + lerp(5.0, 0.0, difficulty) as i32 + cur_tier as i32 * 4;
        let shield_amount = ((i as f32 - shield_start) * shields_per_wave).max(0.0);
        let space = if start == 0 {
            1
        } else {
            rng.random_range_i32(1, 2)
        };
        let ctier = cur_tier;
        let unit = cur_species[ctier.min(cur_species.len() - 1)].to_string();

        out.push(group(unit.clone(), |g| {
            g.unit_amount = if f == start {
                1
            } else {
                6 / scaling[ctier] as i32
            };
            g.begin = f;
            g.end = if f + next >= cap {
                SpawnGroup::NEVER
            } else {
                f + next
            };
            g.max = 13;
            g.unit_scaling = if difficulty < 0.4 {
                rng.random_range_f32(2.5, 5.0)
            } else {
                rng.random_range_f32(1.0, 4.0)
            } * scaling[ctier];
            g.shields = shield_amount;
            g.shield_scaling = shields_per_wave;
            g.spacing = space;
        }));

        out.push(group(unit, |g| {
            g.unit_amount = 3 / scaling[ctier] as i32;
            g.begin = f + next - 1;
            g.end = f + next + rng.random_range_i32(6, 10);
            g.max = 6;
            g.unit_scaling = rng.random_range_f32(2.0, 4.0);
            g.spacing = rng.random_range_i32(2, 4);
            g.shields = shield_amount / 2.0;
            g.shield_scaling = shields_per_wave;
        }));

        i += next + 1;
        if cur_tier < 3 || (rng.chance(0.05) && difficulty > 0.8) {
            cur_tier += 1;
        }
        cur_tier = cur_tier.min(3);
        if rng.chance(0.3) {
            cur_species = random_species(species, rng).clone();
        }
    }
}

fn group(type_name: impl Into<String>, configure: impl FnOnce(&mut SpawnGroup)) -> SpawnGroup {
    let mut group = SpawnGroup::new(type_name);
    configure(&mut group);
    group
}

fn random_species<'a>(species: &'a [Vec<&'a str>], rng: &mut WaveRand) -> &'a Vec<&'a str> {
    &species[rng.random_i32(0, species.len() as i32 - 1) as usize]
}

fn is_air(name: &str) -> bool {
    matches!(name, "flare")
}

fn is_naval(name: &str) -> bool {
    matches!(name, "risso" | "retusa")
}

fn lerp(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

#[derive(Debug, Clone)]
struct WaveRand {
    state: u64,
}

impl WaveRand {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn chance(&mut self, chance: f64) -> bool {
        (self.next_f32() as f64) < chance
    }

    fn random_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + (self.next_u32() % ((max - min + 1) as u32)) as i32
    }

    fn random_range_i32(&mut self, min: i32, max: i32) -> i32 {
        self.random_i32(min, max)
    }

    fn random_range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_spawn_table_matches_upstream_order_and_key_fields() {
        let groups = default_spawn_groups();
        assert_eq!(WAVE_VERSION, 7);
        assert_eq!(groups.len(), 28);

        assert_eq!(groups[0].r#type, "dagger");
        assert_eq!(groups[0].end, 10);
        assert_eq!(groups[0].unit_scaling, 2.0);
        assert_eq!(groups[0].max, 30);

        assert_eq!(groups[8].r#type, "spiroct");
        assert_eq!(groups[8].begin, 45);
        assert_eq!(groups[8].effect.as_deref(), Some("overdrive"));
        assert_eq!(groups[8].shields, 100.0);

        assert_eq!(groups[14].r#type, "nova");
        assert_eq!(groups[14].items, Some(ItemStack::new("blast-compound", 60)));
        assert_eq!(groups[15].items, Some(ItemStack::new("pyratite", 100)));

        assert_eq!(groups[27].r#type, "toxopid");
        assert_eq!(groups[27].begin, 210);
        assert_eq!(groups[27].shields, 1000.0);
        assert_eq!(groups[27].shield_scaling, 35.0);
    }

    #[test]
    fn waves_lazy_get_reuses_default_spawn_groups() {
        let mut waves = Waves::new();
        assert_eq!(waves.get().len(), 28);
        assert_eq!(waves.get()[0].r#type, "dagger");
        assert_eq!(waves.take().len(), 28);
    }

    #[test]
    fn generated_waves_keep_java_shape_with_bosses_shift_and_attack_megas() {
        let generated = generate_with_seed(0.8, 123, true, false, false);
        assert!(generated
            .iter()
            .any(|group| group.effect.as_deref() == Some("boss")));
        assert!(generated.iter().any(|group| group.r#type == "mega"));
        assert!(generated.iter().any(|group| group.begin < 0));
        assert!(generated.iter().any(|group| group.end > 1_000_000_000));
    }

    #[test]
    fn air_only_generation_filters_to_flying_species_starts() {
        let generated = generate_with_seed(0.4, 99, false, true, false);
        assert!(generated.iter().any(|group| group.r#type == "flare"));
        assert!(generated.iter().all(|group| matches!(
            group.r#type.as_str(),
            "flare" | "horizon" | "zenith" | "quad" | "antumbra" | "eclipse"
        )));
    }
}
