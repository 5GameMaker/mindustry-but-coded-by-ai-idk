use crate::mindustry::r#type::Item;

pub fn load() -> Vec<Item> {
    let mut next_id = 0;
    let mut make = |name: &str| {
        let item = Item::new(next_id, name);
        next_id += 1;
        item
    };

    let mut copper = make("copper");
    copper.color_rgba = 0xd99d73ff;
    copper.hardness = 1;
    copper.cost = 0.5;
    copper.always_unlocked();

    let mut lead = make("lead");
    lead.color_rgba = 0x8c7fa9ff;
    lead.hardness = 1;
    lead.cost = 0.7;

    let mut metaglass = make("metaglass");
    metaglass.color_rgba = 0xebeef5ff;
    metaglass.cost = 1.5;

    let mut graphite = make("graphite");
    graphite.color_rgba = 0xb2c6d2ff;
    graphite.cost = 1.0;

    let mut sand = make("sand");
    sand.color_rgba = 0xf7cba4ff;
    sand.low_priority = true;
    sand.buildable = false;
    sand.always_unlocked();

    let mut coal = make("coal");
    coal.color_rgba = 0x272727ff;
    coal.explosiveness = 0.2;
    coal.flammability = 1.0;
    coal.hardness = 2;
    coal.buildable = false;

    let mut titanium = make("titanium");
    titanium.color_rgba = 0x8da1e3ff;
    titanium.hardness = 3;
    titanium.cost = 1.0;

    let mut thorium = make("thorium");
    thorium.color_rgba = 0xf9a3c7ff;
    thorium.explosiveness = 0.2;
    thorium.hardness = 4;
    thorium.radioactivity = 1.0;
    thorium.cost = 1.1;
    thorium.health_scaling = 0.2;

    let mut scrap = make("scrap");
    scrap.color_rgba = 0x777777ff;
    scrap.cost = 0.5;

    let mut silicon = make("silicon");
    silicon.color_rgba = 0x53565cff;
    silicon.cost = 0.8;

    let mut plastanium = make("plastanium");
    plastanium.color_rgba = 0xcbd97fff;
    plastanium.flammability = 0.1;
    plastanium.explosiveness = 0.2;
    plastanium.cost = 1.3;
    plastanium.health_scaling = 0.1;

    let mut phase_fabric = make("phase-fabric");
    phase_fabric.color_rgba = 0xf4ba6eff;
    phase_fabric.cost = 1.3;
    phase_fabric.radioactivity = 0.6;
    phase_fabric.health_scaling = 0.25;

    let mut surge_alloy = make("surge-alloy");
    surge_alloy.color_rgba = 0xf3e979ff;
    surge_alloy.cost = 1.2;
    surge_alloy.charge = 0.75;
    surge_alloy.health_scaling = 0.25;

    let mut spore_pod = make("spore-pod");
    spore_pod.color_rgba = 0x7457ceff;
    spore_pod.flammability = 1.15;
    spore_pod.buildable = false;

    let mut blast_compound = make("blast-compound");
    blast_compound.color_rgba = 0xff795eff;
    blast_compound.flammability = 0.4;
    blast_compound.explosiveness = 1.2;
    blast_compound.buildable = false;

    let mut pyratite = make("pyratite");
    pyratite.color_rgba = 0xffaa5fff;
    pyratite.flammability = 1.4;
    pyratite.explosiveness = 0.4;
    pyratite.buildable = false;

    let mut beryllium = make("beryllium");
    beryllium.color_rgba = 0x3a8f64ff;
    beryllium.hardness = 3;
    beryllium.cost = 1.2;
    beryllium.health_scaling = 0.6;

    let mut tungsten = make("tungsten");
    tungsten.color_rgba = 0x768a9aff;
    tungsten.hardness = 5;
    tungsten.cost = 1.5;
    tungsten.health_scaling = 0.8;

    let mut oxide = make("oxide");
    oxide.color_rgba = 0xe4ffd6ff;
    oxide.cost = 1.2;
    oxide.health_scaling = 0.5;

    let mut carbide = make("carbide");
    carbide.color_rgba = 0x89769aff;
    carbide.cost = 1.4;
    carbide.health_scaling = 1.1;

    let mut fissile_matter = make("fissile-matter");
    fissile_matter.color_rgba = 0x5e988dff;
    fissile_matter.radioactivity = 1.5;
    fissile_matter.hidden = true;

    let mut dormant_cyst = make("dormant-cyst");
    dormant_cyst.color_rgba = 0xdf824dff;
    dormant_cyst.flammability = 0.1;
    dormant_cyst.hidden = true;

    vec![
        scrap,
        copper,
        lead,
        graphite,
        coal,
        titanium,
        thorium,
        silicon,
        plastanium,
        phase_fabric,
        surge_alloy,
        spore_pod,
        sand,
        blast_compound,
        pyratite,
        metaglass,
        beryllium,
        tungsten,
        oxide,
        carbide,
        fissile_matter,
        dormant_cyst,
    ]
}

trait ItemUnlockExt {
    fn always_unlocked(&mut self);
}

impl ItemUnlockExt for Item {
    fn always_unlocked(&mut self) {
        self.base.always_unlocked = true;
        self.base.unlocked = true;
    }
}

#[cfg(test)]
mod tests {
    use super::load;

    #[test]
    fn vanilla_item_vector_order_matches_existing_content_header_order() {
        let items = load();
        let names: Vec<_> = items
            .iter()
            .map(|item| item.base.mappable.name.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "scrap",
                "copper",
                "lead",
                "graphite",
                "coal",
                "titanium",
                "thorium",
                "silicon",
                "plastanium",
                "phase-fabric",
                "surge-alloy",
                "spore-pod",
                "sand",
                "blast-compound",
                "pyratite",
                "metaglass",
                "beryllium",
                "tungsten",
                "oxide",
                "carbide",
                "fissile-matter",
                "dormant-cyst",
            ]
        );
    }

    #[test]
    fn item_content_ids_keep_java_static_field_creation_order() {
        let items = load();
        let id_of = |name: &str| {
            items
                .iter()
                .find(|item| item.base.mappable.name == name)
                .unwrap()
                .base
                .mappable
                .base
                .id
        };
        assert_eq!(id_of("copper"), 0);
        assert_eq!(id_of("lead"), 1);
        assert_eq!(id_of("metaglass"), 2);
        assert_eq!(id_of("scrap"), 8);
        assert_eq!(id_of("dormant-cyst"), 21);
    }

    #[test]
    fn item_core_properties_match_upstream_subset() {
        let items = load();
        let copper = items
            .iter()
            .find(|item| item.base.mappable.name == "copper")
            .unwrap();
        assert_eq!(copper.color_rgba, 0xd99d73ff);
        assert_eq!(copper.hardness, 1);
        assert!(copper.base.unlocked());

        let sand = items
            .iter()
            .find(|item| item.base.mappable.name == "sand")
            .unwrap();
        assert!(!sand.buildable);
        assert!(sand.low_priority);
        assert!(sand.base.unlocked());

        let thorium = items
            .iter()
            .find(|item| item.base.mappable.name == "thorium")
            .unwrap();
        assert_eq!(thorium.radioactivity, 1.0);
        assert_eq!(thorium.hardness, 4);
    }
}
