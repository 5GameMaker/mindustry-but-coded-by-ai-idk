use crate::mindustry::{
    ctype::{Content, ContentId, ContentType},
    r#type::SectorPreset,
};

pub fn load() -> Vec<SectorPreset> {
    let mut next_id = 0;
    let mut presets = Vec::new();

    push(
        &mut presets,
        &mut next_id,
        preset("groundZero", "serpulo", 15)
            .always_unlocked(true)
            .add_starting_items(true)
            .capture_wave(10)
            .difficulty(1.0)
            .override_launch_defaults(false, false)
            .no_lighting(true)
            .start_wave_time_multiplier(3.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("saltFlats", "serpulo", 101).difficulty(5.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("testingGrounds", "serpulo", 3)
            .difficulty(7.0)
            .capture_wave(33),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("frozenForest", "serpulo", 86)
            .capture_wave(15)
            .difficulty(2.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("biomassFacility", "serpulo", 81)
            .capture_wave(20)
            .difficulty(3.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("taintedWoods", "serpulo", 221)
            .capture_wave(33)
            .difficulty(5.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("crateredBattleground", "serpulo", 18)
            .capture_wave(20)
            .difficulty(2.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("ruinousShores", "serpulo", 213)
            .capture_wave(30)
            .difficulty(3.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("perilousHarbor", "serpulo", 47).difficulty(4.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("facility32m", "serpulo", 64)
            .capture_wave(25)
            .difficulty(4.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("windsweptIslands", "serpulo", 246)
            .capture_wave(30)
            .difficulty(4.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("stainedMountains", "serpulo", 20)
            .capture_wave(30)
            .difficulty(3.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("extractionOutpost", "serpulo", 165).difficulty(5.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("coastline", "serpulo", 108)
            .capture_wave(30)
            .difficulty(5.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("weatheredChannels", "serpulo", 39)
            .capture_wave(40)
            .difficulty(9.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("navalFortress", "serpulo", 216).difficulty(8.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("frontier", "serpulo", 50).difficulty(4.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("fungalPass", "serpulo", 21).difficulty(2.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("infestedCanyons", "serpulo", 210).difficulty(4.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("atolls", "serpulo", 1).difficulty(7.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("sunkenPier", "serpulo", -1)
            .capture_wave(50)
            .difficulty(8.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("mycelialBastion", "serpulo", 260).difficulty(8.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("overgrowth", "serpulo", 134).difficulty(5.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("tarFields", "serpulo", 23)
            .capture_wave(40)
            .difficulty(5.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("impact0078", "serpulo", 227)
            .capture_wave(45)
            .difficulty(7.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("desolateRift", "serpulo", 123)
            .capture_wave(18)
            .difficulty(8.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("nuclearComplex", "serpulo", 130)
            .capture_wave(50)
            .difficulty(7.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("littoralShipyard", "serpulo", 204).difficulty(9.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("planetaryTerminal", "serpulo", 93)
            .difficulty(10.0)
            .last_sector(true),
    );

    register_sector_submissions(&mut presets, &mut next_id);

    push(
        &mut presets,
        &mut next_id,
        preset("onset", "erekir", 10)
            .always_unlocked(true)
            .difficulty(1.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("aegis", "erekir", 88).difficulty(3.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("lake", "erekir", 41).difficulty(4.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("intersect", "erekir", 36)
            .difficulty(5.0)
            .capture_wave(9)
            .attack_after_waves(true),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("atlas", "erekir", 14).difficulty(5.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("split", "erekir", 19).difficulty(2.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("basin", "erekir", 29).difficulty(6.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("marsh", "erekir", 25).difficulty(4.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("peaks", "erekir", 30).difficulty(3.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("ravine", "erekir", 39)
            .difficulty(4.0)
            .capture_wave(24),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("caldera-erekir", "erekir", 43).difficulty(4.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("stronghold", "erekir", 18).difficulty(7.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("crevice", "erekir", 3)
            .difficulty(6.0)
            .capture_wave(46),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("siege", "erekir", 58).difficulty(8.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("crossroads", "erekir", 37).difficulty(7.0),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("karst", "erekir", 5)
            .difficulty(9.0)
            .capture_wave(10),
    );
    push(
        &mut presets,
        &mut next_id,
        preset("origin", "erekir", 12)
            .difficulty(10.0)
            .last_sector(true),
    );

    presets
}

fn register_sector_submissions(presets: &mut Vec<SectorPreset>, next_id: &mut ContentId) {
    const HIDDEN: &[(i32, i32, f32)] = &[
        (76, -1, 8.0),
        (47, -1, 0.0),
        (225, -1, 0.0),
        (111, -1, 0.0),
        (176, -1, 0.0),
        (13, -1, 0.0),
        (259, -1, 0.0),
        (192, -1, 0.0),
        (127, -1, 0.0),
        (207, -1, 0.0),
        (94, -1, 0.0),
        (16, -1, 0.0),
        (116, -1, 0.0),
        (69, -1, 0.0),
        (92, -1, 0.0),
        (197, -1, 0.0),
        (67, -1, 0.0),
        (180, -1, 0.0),
        (55, -1, 0.0),
        (19, -1, 0.0),
        (200, -1, 0.0),
        (191, -1, 0.0),
        (6, -1, 9.0),
        (265, -1, 0.0),
        (161, -1, 0.0),
        (24, -1, 0.0),
        (263, -1, 0.0),
        (66, -1, 0.0),
        (248, -1, 0.0),
        (133, -1, 0.0),
        (185, -1, 0.0),
        (254, -1, 0.0),
        (0, -1, 0.0),
        (30, -1, 0.0),
        (20, -1, 0.0),
        (162, -1, 0.0),
        (230, -1, 0.0),
        (240, -1, 8.0),
        (202, 33, 6.0),
        (246, -1, 0.0),
        (244, -1, 0.0),
        (242, -1, 0.0),
        (243, -1, 0.0),
        (247, -1, 0.0),
        (245, -1, 0.0),
    ];

    for (sector_id, capture_wave, difficulty) in HIDDEN {
        push(
            presets,
            next_id,
            hidden_serpulo_sector(*sector_id, *capture_wave, *difficulty),
        );
    }

    push(
        presets,
        next_id,
        hidden_serpulo_sector(27, -1, 10.0)
            .show_hidden(true)
            .shielded_by(246)
            .shielded_by(244)
            .shielded_by(242),
    );

    push(
        presets,
        next_id,
        preset("fallenVessel", "serpulo", -1)
            .require_unlock(false)
            .capture_wave(70)
            .difficulty(9.0),
    );
    push(
        presets,
        next_id,
        preset("geothermalStronghold", "serpulo", 264)
            .require_unlock(false)
            .difficulty(10.0),
    );
    push(
        presets,
        next_id,
        preset("cruxscape", "serpulo", 54)
            .require_unlock(false)
            .difficulty(10.0),
    );
}

fn preset(name: &str, planet_name: &str, sector: i32) -> SectorPreset {
    SectorPreset::with_planet_sector(name, planet_name, sector)
}

fn hidden_serpulo_sector(sector_id: i32, capture_wave: i32, difficulty: f32) -> SectorPreset {
    let mut preset = SectorPreset::with_file_planet_sector(
        format!("sector-serpulo-{sector_id}"),
        format!("hidden/{sector_id}"),
        "serpulo",
        sector_id,
    )
    .require_unlock(false);

    if difficulty > 0.0 {
        preset.difficulty = difficulty;
    }
    if capture_wave > 0 {
        preset.capture_wave = capture_wave;
    }

    preset
}

fn push(presets: &mut Vec<SectorPreset>, next_id: &mut ContentId, preset: SectorPreset) {
    presets.push(preset.with_id(*next_id));
    *next_id += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_sector_preset_order_matches_upstream_loader() {
        let presets = load();
        assert_eq!(presets.len(), 95);
        assert_eq!(
            names(&presets[..4]),
            vec!["groundZero", "saltFlats", "testingGrounds", "frozenForest"]
        );
        assert_eq!(presets[28].name, "planetaryTerminal");
        assert_eq!(presets[29].name, "sector-serpulo-76");
        assert_eq!(presets[73].name, "sector-serpulo-245");
        assert_eq!(presets[74].name, "sector-serpulo-27");
        assert_eq!(presets[75].name, "fallenVessel");
        assert_eq!(presets[78].name, "onset");
        assert_eq!(presets[94].name, "origin");

        for (index, preset) in presets.iter().enumerate() {
            assert_eq!(preset.id(), index as ContentId);
            assert_eq!(preset.content_type(), ContentType::Sector);
        }
    }

    #[test]
    fn serpulo_campaign_presets_keep_upstream_key_fields() {
        let presets = load();
        let ground_zero = by_name(&presets, "groundZero");
        assert_eq!(ground_zero.planet_name.as_deref(), Some("serpulo"));
        assert_eq!(ground_zero.sector_id, Some(15));
        assert!(ground_zero.always_unlocked);
        assert!(ground_zero.add_starting_items);
        assert_eq!(ground_zero.capture_wave, 10);
        assert_eq!(ground_zero.difficulty, 1.0);
        assert!(ground_zero.override_launch_defaults);
        assert!(!ground_zero.allow_launch_schematics);
        assert!(!ground_zero.allow_launch_loadout);
        assert!(ground_zero.no_lighting);
        assert_eq!(ground_zero.start_wave_time_multiplier, 3.0);

        let terminal = by_name(&presets, "planetaryTerminal");
        assert_eq!(terminal.sector_id, Some(93));
        assert_eq!(terminal.difficulty, 10.0);
        assert!(terminal.is_last_sector);

        let sunken = by_name(&presets, "sunkenPier");
        assert_eq!(sunken.original_position, -1);
        assert_eq!(sunken.sector_id, Some(0));
        assert_eq!(sunken.capture_wave, 50);
    }

    #[test]
    fn hidden_sector_submissions_keep_file_name_unlock_and_shield_metadata() {
        let presets = load();
        let first = by_name(&presets, "sector-serpulo-76");
        assert_eq!(first.file_name.as_deref(), Some("hidden/76"));
        assert_eq!(first.planet_name.as_deref(), Some("serpulo"));
        assert_eq!(first.sector_id, Some(76));
        assert!(!first.require_unlock);
        assert_eq!(first.capture_wave, 0);
        assert_eq!(first.difficulty, 8.0);

        let captured = by_name(&presets, "sector-serpulo-202");
        assert_eq!(captured.capture_wave, 33);
        assert_eq!(captured.difficulty, 6.0);

        let shielded = by_name(&presets, "sector-serpulo-27");
        assert!(shielded.show_hidden);
        assert!(!shielded.require_unlock);
        assert_eq!(shielded.difficulty, 10.0);
        assert_eq!(shielded.shield_sector_ids, vec![246, 244, 242]);
    }

    #[test]
    fn extra_hidden_and_erekir_presets_keep_upstream_key_fields() {
        let presets = load();
        let fallen = by_name(&presets, "fallenVessel");
        assert_eq!(fallen.original_position, -1);
        assert_eq!(fallen.sector_id, Some(0));
        assert!(!fallen.require_unlock);
        assert_eq!(fallen.capture_wave, 70);
        assert_eq!(fallen.difficulty, 9.0);

        let onset = by_name(&presets, "onset");
        assert_eq!(onset.planet_name.as_deref(), Some("erekir"));
        assert_eq!(onset.sector_id, Some(10));
        assert!(onset.always_unlocked);
        assert_eq!(onset.difficulty, 1.0);

        let intersect = by_name(&presets, "intersect");
        assert_eq!(intersect.capture_wave, 9);
        assert!(intersect.attack_after_waves);

        let origin = by_name(&presets, "origin");
        assert_eq!(origin.sector_id, Some(12));
        assert_eq!(origin.difficulty, 10.0);
        assert!(origin.is_last_sector);
    }

    fn by_name<'a>(presets: &'a [SectorPreset], name: &str) -> &'a SectorPreset {
        presets
            .iter()
            .find(|preset| preset.name == name)
            .unwrap_or_else(|| panic!("missing preset {name}"))
    }

    fn names(presets: &[SectorPreset]) -> Vec<&str> {
        presets.iter().map(|preset| preset.name.as_str()).collect()
    }
}
