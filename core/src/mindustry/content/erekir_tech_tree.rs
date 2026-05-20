use std::collections::BTreeMap;

use crate::mindustry::{
    game::{ObjectiveKind, TechContentRef, TechNode, TechTree},
    r#type::ItemStack,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErekirRebalanceTarget {
    UnitWeapon { unit: String },
    ItemTurretAmmo { block: String },
    ContinuousLiquidTurretAmmo { block: String },
    ContinuousTurretShootType { block: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErekirRebalancePlan {
    pub damage_multiplier: f32,
    pub targets: Vec<ErekirRebalanceTarget>,
}

pub fn rebalance_plan() -> ErekirRebalancePlan {
    ErekirRebalancePlan {
        damage_multiplier: 0.75,
        targets: vec![
            ErekirRebalanceTarget::UnitWeapon {
                unit: "stell".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "locus".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "precept".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "vanquish".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "conquer".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "merui".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "cleroi".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "anthicus".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "tecta".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "collaris".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "elude".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "avert".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "obviate".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "quell".into(),
            },
            ErekirRebalanceTarget::UnitWeapon {
                unit: "disrupt".into(),
            },
            ErekirRebalanceTarget::ItemTurretAmmo {
                block: "breach".into(),
            },
            ErekirRebalanceTarget::ItemTurretAmmo {
                block: "diffuse".into(),
            },
            ErekirRebalanceTarget::ItemTurretAmmo {
                block: "disperse".into(),
            },
            ErekirRebalanceTarget::ItemTurretAmmo {
                block: "titan".into(),
            },
            ErekirRebalanceTarget::ItemTurretAmmo {
                block: "afflict".into(),
            },
            ErekirRebalanceTarget::ItemTurretAmmo {
                block: "lustre".into(),
            },
            ErekirRebalanceTarget::ItemTurretAmmo {
                block: "scathe".into(),
            },
            ErekirRebalanceTarget::ItemTurretAmmo {
                block: "smite".into(),
            },
            ErekirRebalanceTarget::ContinuousLiquidTurretAmmo {
                block: "sublimate".into(),
            },
            ErekirRebalanceTarget::ContinuousTurretShootType {
                block: "malign".into(),
            },
        ],
    }
}

pub fn cost_multipliers() -> BTreeMap<String, f32> {
    let mut multipliers = BTreeMap::from([
        ("copper".into(), 0.9),
        ("lead".into(), 0.9),
        ("metaglass".into(), 0.9),
        ("graphite".into(), 0.9),
        ("sand".into(), 0.9),
        ("coal".into(), 0.9),
        ("titanium".into(), 0.9),
        ("thorium".into(), 0.9),
        ("scrap".into(), 0.9),
        ("silicon".into(), 0.9),
        ("plastanium".into(), 0.9),
        ("phase-fabric".into(), 0.9),
        ("surge-alloy".into(), 0.9),
        ("spore-pod".into(), 0.9),
        ("pyratite".into(), 0.9),
        ("blast-compound".into(), 0.9),
        ("beryllium".into(), 0.9),
        ("tungsten".into(), 0.9),
        ("oxide".into(), 0.9),
        ("carbide".into(), 0.9),
        ("fissile-matter".into(), 0.9),
        ("dormant-cyst".into(), 0.9),
    ]);

    multipliers.insert("oxide".into(), 0.5);
    multipliers.insert("surge-alloy".into(), 0.7);
    multipliers.insert("carbide".into(), 0.3);
    multipliers.insert("phase-fabric".into(), 0.2);
    multipliers
}

pub fn load() -> TechTree {
    let mut tree = TechTree::new();
    tree.node_root_with_unlock("erekir", block("core-bastion"), true, Vec::new(), |tree| {
        tree.set_context_research_cost_multipliers(cost_multipliers());
        load_distribution(tree);
        load_production_and_power(tree);
        load_defense(tree);
        load_cores(tree);
        load_units(tree);
        load_sectors(tree);
        load_produce_tree(tree);
    });
    tree
}

pub fn find_node<'a>(tree: &'a TechTree, name: &str) -> Option<&'a TechNode> {
    tree.all()
        .iter()
        .find(|node| !node.removed && node.content.name == name)
}

fn load_distribution(tree: &mut TechTree) {
    tree.node_with_objectives(
        block("duct"),
        Vec::new(),
        vec![on_planet("erekir")],
        |tree| {
            tree.node_with_objectives(block("duct-router"), Vec::new(), Vec::new(), |tree| {
                tree.node_with_objectives(block("duct-bridge"), Vec::new(), Vec::new(), |tree| {
                    tree.node_with_objectives(
                        block("armored-duct"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_with_objectives(
                                block("surge-conveyor"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_leaf(block("surge-router"), Vec::new());
                                },
                            );
                        },
                    );

                    tree.node_with_objectives(
                        block("unit-cargo-loader"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_leaf(block("unit-cargo-unload-point"), Vec::new());
                        },
                    );
                });

                tree.node_with_objectives(
                    block("overflow-duct"),
                    Vec::new(),
                    vec![on_sector("aegis")],
                    |tree| {
                        tree.node_leaf(block("underflow-duct"), Vec::new());
                        tree.node_with_objectives(
                            block("reinforced-container"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_leaf(block("duct-unloader"), Vec::new());
                                tree.node_leaf(block("reinforced-vault"), Vec::new());
                            },
                        );
                    },
                );

                tree.node_with_objectives(
                    block("reinforced-message"),
                    Vec::new(),
                    vec![on_sector("aegis")],
                    |tree| {
                        tree.node_with_objectives(
                            block("canvas"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_leaf(block("large-canvas"), Vec::new());
                            },
                        );
                    },
                );
            });

            tree.node_with_objectives(
                block("reinforced-payload-conveyor"),
                Vec::new(),
                vec![on_sector("atlas")],
                |tree| {
                    tree.node_with_objectives(
                        block("payload-mass-driver"),
                        Vec::new(),
                        vec![research(block("silicon-arc-furnace")), on_sector("split")],
                        |tree| {
                            tree.node_with_objectives(
                                block("payload-loader"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_with_objectives(
                                        block("payload-unloader"),
                                        Vec::new(),
                                        Vec::new(),
                                        |tree| {
                                            tree.node_leaf(
                                                block("large-payload-mass-driver"),
                                                Vec::new(),
                                            );
                                        },
                                    );
                                },
                            );

                            tree.node_with_objectives(
                                block("constructor"),
                                Vec::new(),
                                vec![on_sector("split")],
                                |tree| {
                                    tree.node_with_objectives(
                                        block("small-deconstructor"),
                                        Vec::new(),
                                        vec![on_sector("peaks")],
                                        |tree| {
                                            tree.node_with_objectives(
                                                block("large-constructor"),
                                                Vec::new(),
                                                vec![on_sector("siege")],
                                                |_| {},
                                            );
                                            tree.node_with_objectives(
                                                block("deconstructor"),
                                                Vec::new(),
                                                vec![on_sector("siege")],
                                                |_| {},
                                            );
                                        },
                                    );
                                },
                            );
                        },
                    );

                    tree.node_leaf(block("reinforced-payload-router"), Vec::new());
                },
            );
        },
    );
}

fn load_production_and_power(tree: &mut TechTree) {
    tree.node_with_objectives(block("plasma-bore"), Vec::new(), Vec::new(), |tree| {
        tree.node_with_objectives(
            block("impact-drill"),
            Vec::new(),
            vec![on_sector("aegis")],
            |tree| {
                tree.node_with_objectives(
                    block("large-plasma-bore"),
                    Vec::new(),
                    vec![on_sector("caldera-erekir")],
                    |tree| {
                        tree.node_with_objectives(
                            block("eruption-drill"),
                            Vec::new(),
                            vec![on_sector("stronghold")],
                            |_| {},
                        );
                        tree.node_with_objectives(
                            block("large-cliff-crusher"),
                            Vec::new(),
                            vec![on_sector("stronghold")],
                            |_| {},
                        );
                    },
                );
            },
        );
    });

    tree.node_with_objectives(block("turbine-condenser"), Vec::new(), Vec::new(), |tree| {
        tree.node_with_objectives(block("beam-node"), Vec::new(), Vec::new(), |tree| {
            tree.node_with_objectives(
                block("vent-condenser"),
                Vec::new(),
                vec![on_sector("aegis")],
                |tree| {
                    tree.node_with_objectives(
                        block("chemical-combustion-chamber"),
                        Vec::new(),
                        vec![on_sector("basin")],
                        |tree| {
                            tree.node_with_objectives(
                                block("pyrolysis-generator"),
                                Vec::new(),
                                vec![on_sector("crevice")],
                                |tree| {
                                    tree.node_with_objectives(
                                        block("flux-reactor"),
                                        Vec::new(),
                                        vec![
                                            on_sector("crossroads"),
                                            research(block("cyanogen-synthesizer")),
                                        ],
                                        |tree| {
                                            tree.node_with_objectives(
                                                block("neoplasia-reactor"),
                                                Vec::new(),
                                                vec![on_sector("karst")],
                                                |_| {},
                                            );
                                        },
                                    );
                                },
                            );
                        },
                    );
                },
            );

            tree.node_with_objectives(
                block("beam-tower"),
                Vec::new(),
                vec![on_sector("peaks")],
                |tree| {
                    tree.node_with_objectives(
                        block("beam-link"),
                        Vec::new(),
                        vec![on_sector("crossroads")],
                        |_| {},
                    );
                },
            );

            tree.node_with_objectives(
                block("regen-projector"),
                Vec::new(),
                vec![on_sector("peaks")],
                |tree| {
                    tree.node_with_objectives(
                        block("build-tower"),
                        Vec::new(),
                        vec![on_sector("stronghold")],
                        |tree| {
                            tree.node_with_objectives(
                                block("shockwave-tower"),
                                Vec::new(),
                                vec![on_sector("siege")],
                                |_| {},
                            );
                        },
                    );
                },
            );
        });

        tree.node_with_objectives(
            block("reinforced-conduit"),
            Vec::new(),
            vec![on_sector("aegis")],
            |tree| {
                tree.node_with_objectives(
                    block("reinforced-pump"),
                    Vec::new(),
                    vec![on_sector("basin")],
                    |_| {},
                );

                tree.node_with_objectives(
                    block("reinforced-liquid-junction"),
                    Vec::new(),
                    Vec::new(),
                    |tree| {
                        tree.node_leaf(block("reinforced-bridge-conduit"), Vec::new());
                        tree.node_with_objectives(
                            block("reinforced-liquid-router"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_with_objectives(
                                    block("reinforced-liquid-container"),
                                    Vec::new(),
                                    Vec::new(),
                                    |tree| {
                                        tree.node_with_objectives(
                                            block("reinforced-liquid-tank"),
                                            Vec::new(),
                                            vec![sector_complete("intersect")],
                                            |_| {},
                                        );
                                    },
                                );
                            },
                        );
                    },
                );
            },
        );

        tree.node_with_objectives(block("cliff-crusher"), Vec::new(), Vec::new(), |tree| {
            tree.node_with_objectives(
                block("silicon-arc-furnace"),
                Vec::new(),
                Vec::new(),
                |tree| {
                    tree.node_with_objectives(
                        block("electrolyzer"),
                        Vec::new(),
                        vec![on_sector("atlas")],
                        |tree| {
                            tree.node_with_objectives(
                                block("oxidation-chamber"),
                                Vec::new(),
                                vec![research(block("tank-refabricator")), on_sector("marsh")],
                                |tree| {
                                    tree.node_with_objectives(
                                        block("surge-crucible"),
                                        Vec::new(),
                                        vec![on_sector("ravine")],
                                        |_| {},
                                    );
                                    tree.node_with_objectives(
                                        block("heat-redirector"),
                                        Vec::new(),
                                        vec![on_sector("ravine")],
                                        |tree| {
                                            tree.node_with_objectives(
                                                block("electric-heater"),
                                                Vec::new(),
                                                vec![
                                                    on_sector("ravine"),
                                                    research(block("afflict")),
                                                ],
                                                |tree| {
                                                    load_heat_children(tree);
                                                },
                                            );
                                        },
                                    );
                                },
                            );

                            tree.node_with_objectives(
                                block("slag-incinerator"),
                                Vec::new(),
                                vec![on_sector("basin")],
                                |_| {},
                            );
                        },
                    );
                },
            );
        });
    });
}

fn load_heat_children(tree: &mut TechTree) {
    tree.node_with_objectives(
        block("slag-heater"),
        Vec::new(),
        vec![on_sector("caldera-erekir")],
        |_| {},
    );
    tree.node_with_objectives(
        block("atmospheric-concentrator"),
        Vec::new(),
        vec![on_sector("caldera-erekir")],
        |tree| {
            tree.node_with_objectives(
                block("cyanogen-synthesizer"),
                Vec::new(),
                vec![on_sector("siege")],
                |_| {},
            );
        },
    );
    tree.node_with_objectives(
        block("carbide-crucible"),
        Vec::new(),
        vec![on_sector("crevice")],
        |tree| {
            tree.node_with_objectives(
                block("phase-synthesizer"),
                Vec::new(),
                vec![on_sector("karst")],
                |tree| {
                    tree.node_with_objectives(
                        block("phase-heater"),
                        Vec::new(),
                        vec![research(block("phase-synthesizer"))],
                        |_| {},
                    );
                },
            );
        },
    );

    tree.node_with_objectives(block("heat-router"), Vec::new(), Vec::new(), |tree| {
        tree.node_leaf(block("small-heat-redirector"), Vec::new());
    });
}

fn load_defense(tree: &mut TechTree) {
    tree.node_with_objectives(
        block("breach"),
        Vec::new(),
        vec![
            research(block("silicon-arc-furnace")),
            research(block("tank-fabricator")),
        ],
        |tree| {
            tree.node_with_objectives(block("beryllium-wall"), Vec::new(), Vec::new(), |tree| {
                tree.node_leaf(block("beryllium-wall-large"), Vec::new());

                tree.node_with_objectives(block("tungsten-wall"), Vec::new(), Vec::new(), |tree| {
                    tree.node_with_objectives(
                        block("tungsten-wall-large"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_leaf(block("blast-door"), Vec::new());
                        },
                    );

                    tree.node_with_objectives(
                        block("reinforced-surge-wall"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_with_objectives(
                                block("reinforced-surge-wall-large"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_leaf(block("shielded-wall"), Vec::new());
                                },
                            );
                        },
                    );

                    tree.node_with_objectives(
                        block("carbide-wall"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_leaf(block("carbide-wall-large"), Vec::new());
                        },
                    );
                });
            });

            tree.node_with_objectives(
                block("diffuse"),
                Vec::new(),
                vec![on_sector("lake")],
                |tree| {
                    tree.node_with_objectives(
                        block("sublimate"),
                        Vec::new(),
                        vec![on_sector("marsh")],
                        |tree| {
                            tree.node_with_objectives(
                                block("afflict"),
                                Vec::new(),
                                vec![on_sector("ravine")],
                                |tree| {
                                    tree.node_with_objectives(
                                        block("titan"),
                                        Vec::new(),
                                        vec![on_sector("stronghold")],
                                        |tree| {
                                            tree.node_with_objectives(
                                                block("lustre"),
                                                Vec::new(),
                                                vec![on_sector("crevice")],
                                                |tree| {
                                                    tree.node_with_objectives(
                                                        block("smite"),
                                                        Vec::new(),
                                                        vec![on_sector("karst")],
                                                        |_| {},
                                                    );
                                                },
                                            );
                                        },
                                    );
                                },
                            );
                        },
                    );

                    tree.node_with_objectives(
                        block("disperse"),
                        Vec::new(),
                        vec![on_sector("stronghold")],
                        |tree| {
                            tree.node_with_objectives(
                                block("scathe"),
                                Vec::new(),
                                vec![on_sector("siege")],
                                |tree| {
                                    tree.node_with_objectives(
                                        block("malign"),
                                        Vec::new(),
                                        vec![sector_complete("karst")],
                                        |_| {},
                                    );
                                },
                            );
                        },
                    );
                },
            );

            tree.node_with_objectives(
                block("radar"),
                Vec::new(),
                vec![
                    research(block("beam-node")),
                    research(block("turbine-condenser")),
                    research(block("tank-fabricator")),
                    on_sector("aegis"),
                ],
                |_| {},
            );
        },
    );
}

fn load_cores(tree: &mut TechTree) {
    tree.node_with_objectives(
        block("core-citadel"),
        Vec::new(),
        vec![sector_complete("peaks")],
        |tree| {
            tree.node_with_objectives(
                block("core-acropolis"),
                Vec::new(),
                vec![sector_complete("siege")],
                |_| {},
            );
        },
    );
}

fn load_units(tree: &mut TechTree) {
    tree.node_with_objectives(
        block("tank-fabricator"),
        Vec::new(),
        vec![
            research(block("silicon-arc-furnace")),
            research(block("plasma-bore")),
            research(block("turbine-condenser")),
        ],
        |tree| {
            tree.node_leaf(unit("stell"), Vec::new());
            tree.node_with_objectives(
                block("unit-repair-tower"),
                Vec::new(),
                vec![on_sector("ravine"), research(block("mech-refabricator"))],
                |_| {},
            );

            tree.node_with_objectives(
                block("ship-fabricator"),
                Vec::new(),
                vec![on_sector("lake")],
                |tree| {
                    tree.node_leaf(unit("elude"), Vec::new());
                    tree.node_with_objectives(
                        block("mech-fabricator"),
                        Vec::new(),
                        vec![on_sector("intersect")],
                        |tree| {
                            tree.node_leaf(unit("merui"), Vec::new());
                            tree.node_with_objectives(
                                block("tank-refabricator"),
                                Vec::new(),
                                vec![on_sector("atlas")],
                                |tree| {
                                    tree.node_leaf(unit("locus"), Vec::new());
                                    tree.node_with_objectives(
                                        block("mech-refabricator"),
                                        Vec::new(),
                                        vec![on_sector("basin")],
                                        |tree| {
                                            tree.node_leaf(unit("cleroi"), Vec::new());
                                            tree.node_with_objectives(
                                                block("ship-refabricator"),
                                                Vec::new(),
                                                vec![on_sector("peaks")],
                                                |tree| {
                                                    tree.node_leaf(unit("avert"), Vec::new());
                                                    tree.node_with_objectives(
                                                        block("prime-refabricator"),
                                                        Vec::new(),
                                                        vec![on_sector("stronghold")],
                                                        |tree| {
                                                            tree.node_leaf(
                                                                unit("precept"),
                                                                Vec::new(),
                                                            );
                                                            tree.node_leaf(
                                                                unit("anthicus"),
                                                                Vec::new(),
                                                            );
                                                            tree.node_leaf(
                                                                unit("obviate"),
                                                                Vec::new(),
                                                            );
                                                        },
                                                    );

                                                    load_assemblers(tree);
                                                },
                                            );
                                        },
                                    );
                                },
                            );
                        },
                    );
                },
            );
        },
    );
}

fn load_assemblers(tree: &mut TechTree) {
    tree.node_with_objectives(
        block("tank-assembler"),
        Vec::new(),
        vec![
            on_sector("siege"),
            research(block("constructor")),
            research(block("atmospheric-concentrator")),
        ],
        |tree| {
            tree.node_with_objectives(unit("vanquish"), Vec::new(), Vec::new(), |tree| {
                tree.node_with_objectives(
                    unit("conquer"),
                    Vec::new(),
                    vec![on_sector("karst")],
                    |_| {},
                );
            });

            tree.node_with_objectives(
                block("ship-assembler"),
                Vec::new(),
                vec![on_sector("crossroads")],
                |tree| {
                    tree.node_with_objectives(unit("quell"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(
                            unit("disrupt"),
                            Vec::new(),
                            vec![on_sector("karst")],
                            |_| {},
                        );
                    });
                },
            );

            tree.node_with_objectives(
                block("mech-assembler"),
                Vec::new(),
                vec![on_sector("crossroads")],
                |tree| {
                    tree.node_with_objectives(unit("tecta"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(
                            unit("collaris"),
                            Vec::new(),
                            vec![on_sector("karst")],
                            |_| {},
                        );
                    });
                },
            );

            tree.node_with_objectives(
                block("basic-assembler-module"),
                Vec::new(),
                vec![sector_complete("karst")],
                |_| {},
            );
        },
    );
}

fn load_sectors(tree: &mut TechTree) {
    tree.node_with_objectives(sector("onset"), Vec::new(), Vec::new(), |tree| {
        tree.node_with_objectives(
            sector("aegis"),
            Vec::new(),
            vec![
                sector_complete("onset"),
                research(block("duct-router")),
                research(block("duct-bridge")),
            ],
            |tree| {
                tree.node_with_objectives(
                    sector("lake"),
                    Vec::new(),
                    vec![sector_complete("aegis")],
                    |_| {},
                );

                tree.node_with_objectives(
                    sector("intersect"),
                    Vec::new(),
                    vec![
                        sector_complete("aegis"),
                        sector_complete("lake"),
                        research(block("vent-condenser")),
                        research(block("ship-fabricator")),
                    ],
                    |tree| {
                        load_atlas_branch(tree);
                    },
                );
            },
        );
    });
}

fn load_atlas_branch(tree: &mut TechTree) {
    tree.node_with_objectives(
        sector("atlas"),
        Vec::new(),
        vec![
            sector_complete("intersect"),
            research(block("mech-fabricator")),
        ],
        |tree| {
            tree.node_with_objectives(
                sector("split"),
                Vec::new(),
                vec![
                    sector_complete("atlas"),
                    research(block("reinforced-payload-conveyor")),
                    research(block("reinforced-container")),
                ],
                |_| {},
            );

            tree.node_with_objectives(
                sector("basin"),
                Vec::new(),
                vec![sector_complete("atlas")],
                |tree| {
                    tree.node_with_objectives(
                        sector("marsh"),
                        Vec::new(),
                        vec![sector_complete("basin")],
                        |tree| {
                            tree.node_with_objectives(
                                sector("ravine"),
                                Vec::new(),
                                vec![sector_complete("marsh"), research(liquid("slag"))],
                                |tree| {
                                    tree.node_with_objectives(
                                        sector("caldera-erekir"),
                                        Vec::new(),
                                        vec![
                                            sector_complete("peaks"),
                                            sector_complete("ravine"),
                                            research(block("heat-redirector")),
                                        ],
                                        |tree| {
                                            tree.node_with_objectives(
                                                sector("stronghold"),
                                                Vec::new(),
                                                vec![
                                                    sector_complete("caldera-erekir"),
                                                    research(block("core-citadel")),
                                                ],
                                                |tree| {
                                                    tree.node_with_objectives(
                                                        sector("crevice"),
                                                        Vec::new(),
                                                        vec![sector_complete("stronghold")],
                                                        |tree| {
                                                            tree.node_with_objectives(
                                                                sector("siege"),
                                                                Vec::new(),
                                                                vec![sector_complete("crevice")],
                                                                |tree| {
                                                                    tree.node_with_objectives(
                                                                        sector("crossroads"),
                                                                        Vec::new(),
                                                                        vec![sector_complete("siege")],
                                                                        |tree| {
                                                                            tree.node_with_objectives(
                                                                                sector("karst"),
                                                                                Vec::new(),
                                                                                vec![
                                                                                    sector_complete("crossroads"),
                                                                                    research(block("core-acropolis")),
                                                                                ],
                                                                                |tree| {
                                                                                    tree.node_with_objectives(
                                                                                        sector("origin"),
                                                                                        Vec::new(),
                                                                                        vec![
                                                                                            sector_complete("karst"),
                                                                                            research(block("core-acropolis")),
                                                                                            research(unit("vanquish")),
                                                                                            research(unit("disrupt")),
                                                                                            research(unit("collaris")),
                                                                                            research(block("malign")),
                                                                                            research(block("basic-assembler-module")),
                                                                                            research(block("neoplasia-reactor")),
                                                                                        ],
                                                                                        |_| {},
                                                                                    );
                                                                                },
                                                                            );
                                                                        },
                                                                    );
                                                                },
                                                            );
                                                        },
                                                    );
                                                },
                                            );
                                        },
                                    );
                                },
                            );

                            tree.node_with_objectives(
                                sector("peaks"),
                                Vec::new(),
                                vec![sector_complete("marsh"), sector_complete("split")],
                                |_| {},
                            );
                        },
                    );
                },
            );
        },
    );
}

fn load_produce_tree(tree: &mut TechTree) {
    tree.node_produce(item("beryllium"), Vec::new(), Vec::new(), |tree| {
        tree.node_produce(item("sand"), Vec::new(), Vec::new(), |tree| {
            tree.node_produce(item("silicon"), Vec::new(), Vec::new(), |tree| {
                tree.node_produce(item("oxide"), Vec::new(), Vec::new(), |_| {});
            });
        });

        tree.node_produce(liquid("water"), Vec::new(), Vec::new(), |tree| {
            tree.node_produce(liquid("ozone"), Vec::new(), Vec::new(), |tree| {
                tree.node_produce(liquid("hydrogen"), Vec::new(), Vec::new(), |tree| {
                    tree.node_produce(liquid("nitrogen"), Vec::new(), Vec::new(), |_| {});
                    tree.node_produce(liquid("cyanogen"), Vec::new(), Vec::new(), |tree| {
                        tree.node_produce(liquid("neoplasm"), Vec::new(), Vec::new(), |_| {});
                    });
                });
            });
        });

        tree.node_produce(item("graphite"), Vec::new(), Vec::new(), |tree| {
            tree.node_produce(item("tungsten"), Vec::new(), Vec::new(), |tree| {
                tree.node_produce(liquid("slag"), Vec::new(), Vec::new(), |_| {});
                tree.node_produce(liquid("arkycite"), Vec::new(), Vec::new(), |_| {});
                tree.node_produce(item("thorium"), Vec::new(), Vec::new(), |tree| {
                    tree.node_produce(item("carbide"), Vec::new(), Vec::new(), |_| {});
                });
                tree.node_produce(item("surge-alloy"), Vec::new(), Vec::new(), |tree| {
                    tree.node_produce(item("phase-fabric"), Vec::new(), Vec::new(), |_| {});
                });
            });
        });
    });
}

fn block(name: &str) -> TechContentRef {
    TechContentRef::block(name)
}

fn item(name: &str) -> TechContentRef {
    TechContentRef::item(name)
}

fn liquid(name: &str) -> TechContentRef {
    TechContentRef::liquid(name)
}

fn unit(name: &str) -> TechContentRef {
    TechContentRef::unit(name)
}

fn sector(name: &str) -> TechContentRef {
    TechContentRef::sector(name)
}

#[allow(dead_code)]
fn stack(item: &str, amount: i32) -> ItemStack {
    ItemStack::new(item, amount)
}

fn research(content: TechContentRef) -> ObjectiveKind {
    ObjectiveKind::Research(content.objective_content())
}

fn sector_complete(name: &str) -> ObjectiveKind {
    ObjectiveKind::SectorComplete(sector(name).sector_objective_state())
}

fn on_sector(name: &str) -> ObjectiveKind {
    ObjectiveKind::OnSector(sector(name).sector_objective_state())
}

fn on_planet(name: &str) -> ObjectiveKind {
    ObjectiveKind::OnPlanet(TechContentRef::planet(name).into_planet_objective_state())
}

trait PlanetObjectiveExt {
    fn into_planet_objective_state(self) -> crate::mindustry::game::PlanetObjectiveState;
}

impl PlanetObjectiveExt for TechContentRef {
    fn into_planet_objective_state(self) -> crate::mindustry::game::PlanetObjectiveState {
        crate::mindustry::game::PlanetObjectiveState::new(self.name).localized(self.localized_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node<'a>(tree: &'a TechTree, name: &str) -> &'a TechNode {
        find_node(tree, name).unwrap_or_else(|| panic!("missing node {name}"))
    }

    fn child_names(tree: &TechTree, name: &str) -> Vec<String> {
        node(tree, name)
            .children
            .iter()
            .map(|&child| tree.node(child).unwrap().content.name.clone())
            .collect()
    }

    fn objective_strings(tree: &TechTree, name: &str) -> Vec<String> {
        node(tree, name)
            .objectives
            .iter()
            .map(ObjectiveKind::java_to_string)
            .collect()
    }

    #[test]
    fn erekir_root_requires_unlock_and_cost_multipliers_match_upstream() {
        let tree = load();
        let root = tree.node(tree.roots()[0]).unwrap();

        assert_eq!(root.name.as_deref(), Some("erekir"));
        assert_eq!(root.content.name, "core-bastion");
        assert!(root.requires_unlock);
        assert_eq!(root.research_cost_multipliers["beryllium"], 0.9);
        assert_eq!(root.research_cost_multipliers["oxide"], 0.5);
        assert_eq!(root.research_cost_multipliers["surge-alloy"], 0.7);
        assert_eq!(root.research_cost_multipliers["carbide"], 0.3);
        assert_eq!(root.research_cost_multipliers["phase-fabric"], 0.2);
        assert_eq!(
            child_names(&tree, "core-bastion"),
            vec![
                "duct",
                "plasma-bore",
                "turbine-condenser",
                "breach",
                "core-citadel",
                "tank-fabricator",
                "onset",
                "beryllium"
            ]
        );
    }

    #[test]
    fn erekir_rebalance_plan_tracks_units_turrets_and_multiplier() {
        let plan = rebalance_plan();
        assert_eq!(plan.damage_multiplier, 0.75);
        assert!(plan.targets.contains(&ErekirRebalanceTarget::UnitWeapon {
            unit: "stell".into()
        }));
        assert!(plan
            .targets
            .contains(&ErekirRebalanceTarget::ItemTurretAmmo {
                block: "breach".into()
            }));
        assert!(plan
            .targets
            .contains(&ErekirRebalanceTarget::ContinuousLiquidTurretAmmo {
                block: "sublimate".into()
            }));
        assert!(plan
            .targets
            .contains(&ErekirRebalanceTarget::ContinuousTurretShootType {
                block: "malign".into()
            }));
    }

    #[test]
    fn erekir_distribution_and_payload_branch_match_upstream() {
        let tree = load();

        assert_eq!(objective_strings(&tree, "duct"), vec!["onPlanet: erekir"]);
        assert_eq!(
            child_names(&tree, "duct-router"),
            vec!["duct-bridge", "overflow-duct", "reinforced-message"]
        );
        assert_eq!(
            objective_strings(&tree, "overflow-duct"),
            vec!["onSector: aegis"]
        );
        assert_eq!(
            objective_strings(&tree, "payload-mass-driver"),
            vec!["research: silicon-arc-furnace", "onSector: split"]
        );
        assert_eq!(
            child_names(&tree, "payload-mass-driver"),
            vec!["payload-loader", "constructor"]
        );
    }

    #[test]
    fn erekir_production_power_and_defense_objectives_match_upstream() {
        let tree = load();

        assert_eq!(
            objective_strings(&tree, "impact-drill"),
            vec!["onSector: aegis"]
        );
        assert_eq!(
            objective_strings(&tree, "flux-reactor"),
            vec!["onSector: crossroads", "research: cyanogen-synthesizer"]
        );
        assert_eq!(
            objective_strings(&tree, "electric-heater"),
            vec!["onSector: ravine", "research: afflict"]
        );
        assert_eq!(
            objective_strings(&tree, "breach"),
            vec!["research: silicon-arc-furnace", "research: tank-fabricator"]
        );
        assert_eq!(
            objective_strings(&tree, "malign"),
            vec!["sectorComplete: karst"]
        );
        assert_eq!(
            objective_strings(&tree, "core-citadel"),
            vec!["sectorComplete: peaks"]
        );
    }

    #[test]
    fn erekir_unit_and_sector_branches_match_upstream() {
        let tree = load();

        assert_eq!(
            objective_strings(&tree, "tank-fabricator"),
            vec![
                "research: silicon-arc-furnace",
                "research: plasma-bore",
                "research: turbine-condenser"
            ]
        );
        assert_eq!(
            child_names(&tree, "tank-fabricator"),
            vec!["stell", "unit-repair-tower", "ship-fabricator"]
        );
        assert_eq!(
            objective_strings(&tree, "tank-assembler"),
            vec![
                "onSector: siege",
                "research: constructor",
                "research: atmospheric-concentrator"
            ]
        );
        assert_eq!(
            objective_strings(&tree, "aegis"),
            vec![
                "sectorComplete: onset",
                "research: duct-router",
                "research: duct-bridge"
            ]
        );
        assert_eq!(
            objective_strings(&tree, "origin"),
            vec![
                "sectorComplete: karst",
                "research: core-acropolis",
                "research: vanquish",
                "research: disrupt",
                "research: collaris",
                "research: malign",
                "research: basic-assembler-module",
                "research: neoplasia-reactor"
            ]
        );
    }

    #[test]
    fn erekir_produce_tree_keeps_resource_order() {
        let tree = load();

        assert_eq!(
            child_names(&tree, "beryllium"),
            vec!["sand", "water", "graphite"]
        );
        assert_eq!(child_names(&tree, "sand"), vec!["silicon"]);
        assert_eq!(child_names(&tree, "silicon"), vec!["oxide"]);
        assert_eq!(child_names(&tree, "hydrogen"), vec!["nitrogen", "cyanogen"]);
        assert_eq!(
            child_names(&tree, "tungsten"),
            vec!["slag", "arkycite", "thorium", "surge-alloy"]
        );
    }
}
