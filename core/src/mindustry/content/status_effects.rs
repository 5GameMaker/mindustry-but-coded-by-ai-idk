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

    statuses
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
}
