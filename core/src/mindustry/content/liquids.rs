use crate::mindustry::r#type::Liquid;

pub fn load() -> Vec<Liquid> {
    let mut next_id = 0;
    let mut make = |name: &str| {
        let liquid = Liquid::new(next_id, name);
        next_id += 1;
        liquid
    };

    let mut water = make("water");
    water.color_rgba = 0x596ab8ff;
    water.heat_capacity = 0.4;
    water.effect = Some("wet".to_string());
    water.boil_point = 0.5;
    water.gas_color_rgba = 0xe6e6e6ff;
    water.base.always_unlocked = true;
    water.base.unlocked = true;

    let mut slag = make("slag");
    slag.color_rgba = 0xffa166ff;
    slag.temperature = 1.0;
    slag.viscosity = 0.7;
    slag.effect = Some("melting".to_string());
    slag.light_color_rgba = 0xf0511d66;

    let mut oil = make("oil");
    oil.color_rgba = 0x313131ff;
    oil.bar_color_rgba = Some(0x6b675fff);
    oil.flammability = 1.2;
    oil.explosiveness = 1.2;
    oil.heat_capacity = 0.7;
    oil.viscosity = 0.75;
    oil.effect = Some("tarred".to_string());
    oil.boil_point = 0.65;
    oil.gas_color_rgba = 0x666666ff;

    let mut cryofluid = make("cryofluid");
    cryofluid.color_rgba = 0x6ecdecff;
    cryofluid.heat_capacity = 0.9;
    cryofluid.temperature = 0.25;
    cryofluid.effect = Some("freezing".to_string());
    cryofluid.light_color_rgba = 0x0097f533;
    cryofluid.boil_point = 0.55;
    cryofluid.gas_color_rgba = 0xc1e8f5ff;

    let mut neoplasm = make("neoplasm");
    neoplasm.color_rgba = 0xc33e2bff;
    neoplasm.heat_capacity = 0.4;
    neoplasm.temperature = 0.54;
    neoplasm.viscosity = 0.85;
    neoplasm.flammability = 0.0;
    neoplasm.cap_puddles = false;
    neoplasm.move_through_blocks = true;
    neoplasm.incinerable = false;
    neoplasm.block_reactive = false;

    let mut arkycite = make("arkycite");
    arkycite.color_rgba = 0x84a94bff;
    arkycite.flammability = 0.4;
    arkycite.viscosity = 0.7;

    let mut gallium = make("gallium");
    gallium.color_rgba = 0x9a9dbfff;
    gallium.coolant = false;
    gallium.hidden = true;

    let mut ozone = make("ozone");
    ozone.color_rgba = 0xfc81ddff;
    ozone.gas = true;
    ozone.bar_color_rgba = Some(0xd699f0ff);
    ozone.explosiveness = 1.0;
    ozone.flammability = 1.0;

    let mut hydrogen = make("hydrogen");
    hydrogen.color_rgba = 0x9eabf7ff;
    hydrogen.gas = true;
    hydrogen.flammability = 1.0;

    let mut nitrogen = make("nitrogen");
    nitrogen.color_rgba = 0xefe3ffff;
    nitrogen.gas = true;

    let mut cyanogen = make("cyanogen");
    cyanogen.color_rgba = 0x89e8b6ff;
    cyanogen.gas = true;
    cyanogen.flammability = 2.0;

    vec![
        water, slag, oil, cryofluid, neoplasm, arkycite, gallium, ozone, hydrogen, nitrogen,
        cyanogen,
    ]
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn vanilla_liquid_ids_follow_upstream_registration_order() {
        let liquids = load();
        let names: Vec<_> = liquids
            .iter()
            .map(|liquid| liquid.base.mappable.name.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "water",
                "slag",
                "oil",
                "cryofluid",
                "neoplasm",
                "arkycite",
                "gallium",
                "ozone",
                "hydrogen",
                "nitrogen",
                "cyanogen",
            ]
        );
        for (index, liquid) in liquids.iter().enumerate() {
            assert_eq!(liquid.base.mappable.base.id, index as i16);
        }
    }

    #[test]
    fn liquid_core_properties_match_upstream_subset() {
        let liquids = load();
        let water = &liquids[0];
        assert_eq!(water.color_rgba, 0x596ab8ff);
        assert_eq!(water.heat_capacity, 0.4);
        assert_eq!(water.effect.as_deref(), Some("wet"));
        assert!(water.base.unlocked());

        let oil = &liquids[2];
        assert_eq!(oil.viscosity, 0.75);
        assert_eq!(oil.flammability, 1.2);
        assert_eq!(oil.explosiveness, 1.2);

        let neoplasm = &liquids[4];
        assert!(!neoplasm.cap_puddles);
        assert!(neoplasm.move_through_blocks);
        assert!(!neoplasm.incinerable);
        assert!(!neoplasm.block_reactive);

        let cyanogen = &liquids[10];
        assert!(cyanogen.gas);
        assert_eq!(cyanogen.flammability, 2.0);
    }
}
