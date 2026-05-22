use crate::mindustry::r#type::StatusEffect;

pub fn load() -> Vec<StatusEffect> {
    let mut next_id = 0;
    let mut make = |name: &str| {
        let status = StatusEffect::new(next_id, name);
        next_id += 1;
        status
    };

    let mut statuses = Vec::with_capacity(23);

    let none = make("none");
    statuses.push(none);

    // Upstream also wires affinity/opposite reactions and FX for several of
    // these entries. The Rust content registry currently stores only the
    // scalar vanilla metadata, so those hooks are intentionally omitted here.

    let mut burning = make("burning");
    burning.damage = 0.167;
    burning.transition_damage = 8.0;
    burning.color_rgba = 0xffc455ff;
    statuses.push(burning);

    let mut freezing = make("freezing");
    freezing.speed_multiplier = 0.6;
    freezing.health_multiplier = 0.8;
    freezing.transition_damage = 18.0;
    freezing.color_rgba = 0x6ecdecff;
    statuses.push(freezing);

    let mut unmoving = make("unmoving");
    unmoving.speed_multiplier = 0.0;
    unmoving.color_rgba = 0x454545ff;
    statuses.push(unmoving);

    let mut slow = make("slow");
    slow.speed_multiplier = 0.4;
    slow.show = false;
    slow.color_rgba = 0xa2a2a2ff;
    statuses.push(slow);

    let mut fast = make("fast");
    fast.speed_multiplier = 1.6;
    fast.color_rgba = 0xffad4dff;
    statuses.push(fast);

    let mut wet = make("wet");
    wet.speed_multiplier = 0.94;
    wet.transition_damage = 14.0;
    wet.effect_chance = 0.09;
    wet.color_rgba = 0x596ab8ff;
    statuses.push(wet);

    let mut muddy = make("muddy");
    muddy.speed_multiplier = 0.94;
    muddy.effect_chance = 0.09;
    muddy.show = false;
    muddy.color_rgba = 0x432722ff;
    statuses.push(muddy);

    let mut melting = make("melting");
    melting.speed_multiplier = 0.8;
    melting.health_multiplier = 0.8;
    melting.damage = 0.3;
    melting.color_rgba = 0xffa166ff;
    statuses.push(melting);

    let mut sapped = make("sapped");
    sapped.speed_multiplier = 0.7;
    sapped.health_multiplier = 0.8;
    sapped.effect_chance = 0.1;
    sapped.color_rgba = 0x665c9fff;
    statuses.push(sapped);

    let mut electrified = make("electrified");
    electrified.speed_multiplier = 0.7;
    electrified.reload_multiplier = 0.6;
    electrified.effect_chance = 0.1;
    electrified.color_rgba = 0x98ffa9ff;
    statuses.push(electrified);

    let mut spore_slowed = make("spore-slowed");
    spore_slowed.speed_multiplier = 0.8;
    spore_slowed.effect_chance = 0.04;
    spore_slowed.color_rgba = 0x7457ceff;
    statuses.push(spore_slowed);

    let mut tarred = make("tarred");
    tarred.speed_multiplier = 0.6;
    tarred.color_rgba = 0x313131ff;
    statuses.push(tarred);

    let mut overdrive = make("overdrive");
    overdrive.health_multiplier = 0.95;
    overdrive.speed_multiplier = 1.15;
    overdrive.damage_multiplier = 1.4;
    overdrive.damage = -0.01;
    overdrive.permanent = true;
    overdrive.color_rgba = 0xffd37fff;
    statuses.push(overdrive);

    let mut overclock = make("overclock");
    overclock.speed_multiplier = 1.15;
    overclock.damage_multiplier = 1.15;
    overclock.reload_multiplier = 1.25;
    overclock.effect_chance = 0.07;
    overclock.color_rgba = 0xffd37fff;
    statuses.push(overclock);

    let mut shielded = make("shielded");
    shielded.health_multiplier = 3.0;
    shielded.show = false;
    shielded.color_rgba = 0xffd37fff;
    statuses.push(shielded);

    let mut boss = make("boss");
    boss.damage_multiplier = 1.3;
    boss.health_multiplier = 1.5;
    boss.permanent = true;
    boss.color_rgba = 0xf25555ff;
    statuses.push(boss);

    let mut shocked = make("shocked");
    shocked.reactive = true;
    shocked.color_rgba = 0xa9d8ffff;
    statuses.push(shocked);

    let mut blasted = make("blasted");
    blasted.reactive = true;
    blasted.color_rgba = 0xff795eff;
    statuses.push(blasted);

    let mut corroded = make("corroded");
    corroded.interval_damage = 20.0;
    corroded.interval_damage_time = 15.0;
    corroded.interval_damage_pierce = false;
    corroded.effect_chance = 0.1;
    corroded.color_rgba = 0xe4ffd6ff;
    statuses.push(corroded);

    let mut disarmed = make("disarmed");
    disarmed.disarm = true;
    disarmed.show = false;
    disarmed.color_rgba = 0xe9ead3ff;
    statuses.push(disarmed);

    let mut invincible = make("invincible");
    invincible.health_multiplier = f32::INFINITY;
    invincible.show = false;
    statuses.push(invincible);

    let mut dynamic = make("dynamic");
    dynamic.show = false;
    dynamic.dynamic = true;
    dynamic.permanent = true;
    statuses.push(dynamic);

    wire_status_effect_relationships(&mut statuses);
    statuses
}

fn wire_status_effect_relationships(statuses: &mut [StatusEffect]) {
    wire_opposite(statuses, "burning", "wet");
    wire_opposite(statuses, "burning", "freezing");
    wire_affinity(statuses, "burning", "tarred");

    wire_opposite(statuses, "freezing", "melting");
    wire_opposite(statuses, "freezing", "burning");
    wire_affinity(statuses, "freezing", "blasted");

    wire_opposite(statuses, "slow", "fast");
    wire_opposite(statuses, "fast", "slow");

    wire_affinity(statuses, "wet", "shocked");
    wire_opposite(statuses, "wet", "burning");
    wire_opposite(statuses, "wet", "melting");

    wire_opposite(statuses, "melting", "wet");
    wire_opposite(statuses, "melting", "freezing");
    wire_affinity(statuses, "melting", "tarred");

    wire_affinity(statuses, "tarred", "melting");
    wire_affinity(statuses, "tarred", "burning");
}

fn wire_affinity(statuses: &mut [StatusEffect], source_name: &str, target_name: &str) {
    let Some((source_index, target_index)) =
        status_pair_indices(statuses, source_name, target_name)
    else {
        return;
    };

    let source_name = statuses[source_index].name().to_string();
    let target_name = statuses[target_index].name().to_string();
    let Some((source, target)) = pair_mut(statuses, source_index, target_index) else {
        return;
    };

    source.add_affinity(target_name);
    push_unique_name(&mut target.affinities, source_name);
}

fn wire_opposite(statuses: &mut [StatusEffect], first_name: &str, second_name: &str) {
    let Some((first_index, second_index)) = status_pair_indices(statuses, first_name, second_name)
    else {
        return;
    };

    let first_name = statuses[first_index].name().to_string();
    let second_name = statuses[second_index].name().to_string();
    let Some((first, second)) = pair_mut(statuses, first_index, second_index) else {
        return;
    };

    first.add_opposite(second_name);
    second.add_opposite(first_name);
}

fn status_pair_indices(
    statuses: &[StatusEffect],
    first_name: &str,
    second_name: &str,
) -> Option<(usize, usize)> {
    let first = status_index(statuses, first_name)?;
    let second = status_index(statuses, second_name)?;
    Some((first, second))
}

fn status_index(statuses: &[StatusEffect], name: &str) -> Option<usize> {
    statuses.iter().position(|status| status.name() == name)
}

fn pair_mut<T>(values: &mut [T], first: usize, second: usize) -> Option<(&mut T, &mut T)> {
    if first == second || first >= values.len() || second >= values.len() {
        return None;
    }

    if first < second {
        let (left, right) = values.split_at_mut(second);
        Some((&mut left[first], &mut right[0]))
    } else {
        let (left, right) = values.split_at_mut(first);
        Some((&mut right[0], &mut left[second]))
    }
}

fn push_unique_name(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use super::load;

    fn status<'a>(
        statuses: &'a [crate::mindustry::r#type::StatusEffect],
        name: &str,
    ) -> &'a crate::mindustry::r#type::StatusEffect {
        statuses
            .iter()
            .find(|status| status.base.mappable.name == name)
            .unwrap_or_else(|| panic!("missing status effect: {name}"))
    }

    fn sorted_names(values: &[String]) -> Vec<&str> {
        let mut values = values.iter().map(String::as_str).collect::<Vec<_>>();
        values.sort_unstable();
        values
    }

    #[test]
    fn vanilla_status_effects_keep_upstream_order_and_ids() {
        let statuses = load();
        let names: Vec<&str> = statuses
            .iter()
            .map(|status| status.base.mappable.name.as_str())
            .collect();

        assert_eq!(
            names,
            vec![
                "none",
                "burning",
                "freezing",
                "unmoving",
                "slow",
                "fast",
                "wet",
                "muddy",
                "melting",
                "sapped",
                "electrified",
                "spore-slowed",
                "tarred",
                "overdrive",
                "overclock",
                "shielded",
                "boss",
                "shocked",
                "blasted",
                "corroded",
                "disarmed",
                "invincible",
                "dynamic",
            ]
        );

        for (idx, status) in statuses.iter().enumerate() {
            assert_eq!(
                status.base.mappable.base.id, idx as i16,
                "id mismatch for {}",
                status.base.mappable.name
            );
        }
    }

    #[test]
    fn core_vanilla_status_effect_fields_match_upstream_data() {
        let statuses = load();

        let burning = status(&statuses, "burning");
        assert_eq!(burning.damage, 0.167);
        assert_eq!(burning.transition_damage, 8.0);
        assert_eq!(burning.color_rgba, 0xffc455ff);

        let freezing = status(&statuses, "freezing");
        assert_eq!(freezing.speed_multiplier, 0.6);
        assert_eq!(freezing.health_multiplier, 0.8);
        assert_eq!(freezing.transition_damage, 18.0);
        assert_eq!(freezing.color_rgba, 0x6ecdecff);

        let wet = status(&statuses, "wet");
        assert_eq!(wet.speed_multiplier, 0.94);
        assert_eq!(wet.transition_damage, 14.0);
        assert_eq!(wet.effect_chance, 0.09);
        assert_eq!(wet.color_rgba, 0x596ab8ff);

        let melting = status(&statuses, "melting");
        assert_eq!(melting.speed_multiplier, 0.8);
        assert_eq!(melting.health_multiplier, 0.8);
        assert_eq!(melting.damage, 0.3);
        assert_eq!(melting.color_rgba, 0xffa166ff);

        let tarred = status(&statuses, "tarred");
        assert_eq!(tarred.speed_multiplier, 0.6);
        assert_eq!(tarred.color_rgba, 0x313131ff);

        let overdrive = status(&statuses, "overdrive");
        assert_eq!(overdrive.health_multiplier, 0.95);
        assert_eq!(overdrive.speed_multiplier, 1.15);
        assert_eq!(overdrive.damage_multiplier, 1.4);
        assert_eq!(overdrive.damage, -0.01);
        assert!(overdrive.permanent);

        let shielded = status(&statuses, "shielded");
        assert_eq!(shielded.health_multiplier, 3.0);
        assert!(!shielded.show);

        let corroded = status(&statuses, "corroded");
        assert_eq!(corroded.interval_damage, 20.0);
        assert_eq!(corroded.interval_damage_time, 15.0);
        assert!(!corroded.interval_damage_pierce);

        let boss = status(&statuses, "boss");
        assert_eq!(boss.damage_multiplier, 1.3);
        assert_eq!(boss.health_multiplier, 1.5);
        assert!(boss.permanent);

        let shocked = status(&statuses, "shocked");
        assert!(shocked.reactive);

        let disarmed = status(&statuses, "disarmed");
        assert!(disarmed.disarm);
        assert!(!disarmed.show);

        let invincible = status(&statuses, "invincible");
        assert!(invincible.health_multiplier.is_infinite());
        assert!(!invincible.show);

        let dynamic = status(&statuses, "dynamic");
        assert!(dynamic.dynamic);
        assert!(dynamic.permanent);
        assert!(!dynamic.show);
    }

    #[test]
    fn vanilla_status_effect_affinities_and_opposites_match_upstream_init_blocks() {
        let statuses = load();

        assert_eq!(
            sorted_names(&status(&statuses, "burning").opposites),
            vec!["freezing", "wet"]
        );
        assert_eq!(
            sorted_names(&status(&statuses, "burning").affinities),
            vec!["tarred"]
        );

        assert_eq!(
            sorted_names(&status(&statuses, "freezing").opposites),
            vec!["burning", "melting"]
        );
        assert_eq!(
            sorted_names(&status(&statuses, "freezing").affinities),
            vec!["blasted"]
        );
        assert_eq!(
            sorted_names(&status(&statuses, "blasted").affinities),
            vec!["freezing"]
        );

        assert_eq!(
            sorted_names(&status(&statuses, "slow").opposites),
            vec!["fast"]
        );
        assert_eq!(
            sorted_names(&status(&statuses, "fast").opposites),
            vec!["slow"]
        );

        assert_eq!(
            sorted_names(&status(&statuses, "wet").opposites),
            vec!["burning", "melting"]
        );
        assert_eq!(
            sorted_names(&status(&statuses, "wet").affinities),
            vec!["shocked"]
        );
        assert_eq!(
            sorted_names(&status(&statuses, "shocked").affinities),
            vec!["wet"]
        );

        assert_eq!(
            sorted_names(&status(&statuses, "melting").opposites),
            vec!["freezing", "wet"]
        );
        assert_eq!(
            sorted_names(&status(&statuses, "melting").affinities),
            vec!["tarred"]
        );
        assert_eq!(
            sorted_names(&status(&statuses, "tarred").affinities),
            vec!["burning", "melting"]
        );
    }

    #[test]
    fn vanilla_status_effect_transition_directions_match_java_affinity_and_opposite_rules() {
        let statuses = load();

        assert!(status(&statuses, "burning").reacts_with_name("wet"));
        assert!(status(&statuses, "wet").reacts_with_name("burning"));
        assert!(status(&statuses, "freezing").reacts_with_name("melting"));
        assert!(status(&statuses, "melting").reacts_with_name("freezing"));
        assert!(status(&statuses, "slow").reacts_with_name("fast"));
        assert!(status(&statuses, "fast").reacts_with_name("slow"));

        assert!(status(&statuses, "burning").reacts_with_name("tarred"));
        assert!(status(&statuses, "tarred").reacts_with_name("burning"));
        assert!(status(&statuses, "melting").reacts_with_name("tarred"));
        assert!(status(&statuses, "tarred").reacts_with_name("melting"));

        assert!(status(&statuses, "freezing").reacts_with_name("blasted"));
        assert!(!status(&statuses, "blasted").reacts_with_name("freezing"));
        assert!(status(&statuses, "wet").reacts_with_name("shocked"));
        assert!(!status(&statuses, "shocked").reacts_with_name("wet"));
    }
}
