use crate::mindustry::{
    game::{ObjectiveKind, TechContentRef, TechNode, TechTree},
    r#type::ItemStack,
};

pub fn load() -> TechTree {
    let mut tree = TechTree::new();
    tree.node_root("serpulo", block("core-shard"), Vec::new(), |tree| {
        load_distribution(tree);
        load_cores(tree);
        load_production_power_and_logic(tree);
        load_defense(tree);
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
    tree.node_with_objectives(block("conveyor"), Vec::new(), Vec::new(), |tree| {
        tree.node_with_objectives(block("junction"), Vec::new(), Vec::new(), |tree| {
            tree.node_with_objectives(block("router"), Vec::new(), Vec::new(), |tree| {
                tree.node_with_objectives(
                    block("advanced-launch-pad"),
                    Vec::new(),
                    vec![sector_complete("extractionOutpost")],
                    |tree| {
                        tree.node_with_objectives(
                            block("landing-pad"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_with_objectives(
                                    block("interplanetary-accelerator"),
                                    Vec::new(),
                                    vec![sector_complete("planetaryTerminal")],
                                    |_| {},
                                );
                            },
                        );
                    },
                );

                tree.node_leaf(block("distributor"), Vec::new());
                tree.node_with_objectives(block("sorter"), Vec::new(), Vec::new(), |tree| {
                    tree.node_leaf(block("inverted-sorter"), Vec::new());
                    tree.node_with_objectives(
                        block("overflow-gate"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_leaf(block("underflow-gate"), Vec::new());
                        },
                    );
                });
                tree.node_with_objectives(
                    block("container"),
                    Vec::new(),
                    vec![sector_complete("biomassFacility")],
                    |tree| {
                        tree.node_leaf(block("unloader"), Vec::new());
                        tree.node_with_objectives(
                            block("vault"),
                            Vec::new(),
                            vec![sector_complete("stainedMountains")],
                            |_| {},
                        );
                    },
                );

                tree.node_with_objectives(block("item-bridge"), Vec::new(), Vec::new(), |tree| {
                    tree.node_with_objectives(
                        block("titanium-conveyor"),
                        Vec::new(),
                        vec![sector_complete("crateredBattleground")],
                        |tree| {
                            tree.node_with_objectives(
                                block("phase-conveyor"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_with_objectives(
                                        block("mass-driver"),
                                        Vec::new(),
                                        vec![sector_complete("tarFields")],
                                        |_| {},
                                    );
                                },
                            );

                            tree.node_with_objectives(
                                block("payload-conveyor"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_leaf(block("payload-router"), Vec::new());
                                },
                            );

                            tree.node_with_objectives(
                                block("armored-conveyor"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_leaf(block("plastanium-conveyor"), Vec::new());
                                },
                            );
                        },
                    );
                });
            });
        });
    });
}

fn load_cores(tree: &mut TechTree) {
    tree.node_with_objectives(block("core-foundation"), Vec::new(), Vec::new(), |tree| {
        tree.node_leaf(block("core-nucleus"), Vec::new());
    });
}

fn load_production_power_and_logic(tree: &mut TechTree) {
    tree.node_with_objectives(block("mechanical-drill"), Vec::new(), Vec::new(), |tree| {
        tree.node_with_objectives(block("mechanical-pump"), Vec::new(), Vec::new(), |tree| {
            tree.node_with_objectives(block("conduit"), Vec::new(), Vec::new(), |tree| {
                tree.node_with_objectives(
                    block("liquid-junction"),
                    Vec::new(),
                    Vec::new(),
                    |tree| {
                        tree.node_with_objectives(
                            block("liquid-router"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_with_objectives(
                                    block("liquid-container"),
                                    Vec::new(),
                                    Vec::new(),
                                    |tree| {
                                        tree.node_leaf(block("liquid-tank"), Vec::new());
                                    },
                                );

                                tree.node_leaf(block("bridge-conduit"), Vec::new());

                                tree.node_with_objectives(
                                    block("pulse-conduit"),
                                    Vec::new(),
                                    vec![sector_complete("windsweptIslands")],
                                    |tree| {
                                        tree.node_leaf(block("phase-conduit"), Vec::new());
                                        tree.node_leaf(block("plated-conduit"), Vec::new());
                                        tree.node_with_objectives(
                                            block("rotary-pump"),
                                            Vec::new(),
                                            Vec::new(),
                                            |tree| {
                                                tree.node_leaf(block("impulse-pump"), Vec::new());
                                            },
                                        );
                                    },
                                );
                            },
                        );
                    },
                );
            });
        });

        tree.node_with_objectives(block("graphite-press"), Vec::new(), Vec::new(), |tree| {
            tree.node_with_objectives(
                block("pneumatic-drill"),
                Vec::new(),
                vec![sector_complete("frozenForest")],
                |tree| {
                    tree.node_with_objectives(
                        block("cultivator"),
                        Vec::new(),
                        vec![sector_complete("biomassFacility")],
                        |_| {},
                    );

                    tree.node_with_objectives(
                        block("laser-drill"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_with_objectives(
                                block("blast-drill"),
                                Vec::new(),
                                vec![sector_complete("nuclearComplex")],
                                |_| {},
                            );

                            tree.node_with_objectives(
                                block("water-extractor"),
                                Vec::new(),
                                vec![sector_complete("saltFlats")],
                                |tree| {
                                    tree.node_leaf(block("oil-extractor"), Vec::new());
                                },
                            );
                        },
                    );
                },
            );

            tree.node_with_objectives(
                block("pyratite-mixer"),
                Vec::new(),
                vec![sector_complete("crateredBattleground")],
                |tree| {
                    tree.node_with_objectives(
                        block("blast-mixer"),
                        Vec::new(),
                        vec![sector_complete("facility32m")],
                        |_| {},
                    );
                },
            );

            tree.node_with_objectives(
                block("silicon-smelter"),
                Vec::new(),
                vec![sector_complete("frozenForest")],
                |tree| {
                    tree.node_with_objectives(
                        block("spore-press"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_with_objectives(
                                block("coal-centrifuge"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_with_objectives(
                                        block("multi-press"),
                                        Vec::new(),
                                        Vec::new(),
                                        |tree| {
                                            tree.node_leaf(block("silicon-crucible"), Vec::new());
                                        },
                                    );
                                },
                            );

                            tree.node_with_objectives(
                                block("plastanium-compressor"),
                                Vec::new(),
                                vec![sector_complete("windsweptIslands")],
                                |tree| {
                                    tree.node_with_objectives(
                                        block("phase-weaver"),
                                        Vec::new(),
                                        vec![sector_complete("impact0078")],
                                        |_| {},
                                    );
                                },
                            );
                        },
                    );

                    tree.node_with_objectives(
                        block("kiln"),
                        Vec::new(),
                        vec![on_sector("crateredBattleground")],
                        |tree| {
                            tree.node_with_objectives(
                                block("pulverizer"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_with_objectives(
                                        block("incinerator"),
                                        Vec::new(),
                                        Vec::new(),
                                        |tree| {
                                            tree.node_with_objectives(
                                                block("melter"),
                                                Vec::new(),
                                                Vec::new(),
                                                |tree| {
                                                    tree.node_with_objectives(
                                                        block("surge-smelter"),
                                                        Vec::new(),
                                                        vec![sector_complete("coastline")],
                                                        |_| {},
                                                    );

                                                    tree.node_with_objectives(
                                                        block("separator"),
                                                        Vec::new(),
                                                        Vec::new(),
                                                        |tree| {
                                                            tree.node_leaf(
                                                                block("disassembler"),
                                                                Vec::new(),
                                                            );
                                                        },
                                                    );

                                                    tree.node_leaf(
                                                        block("cryofluid-mixer"),
                                                        Vec::new(),
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
                        block("micro-processor"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_with_objectives(
                                block("switch"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_with_objectives(
                                        block("message"),
                                        Vec::new(),
                                        Vec::new(),
                                        |tree| {
                                            tree.node_with_objectives(
                                                block("logic-display"),
                                                Vec::new(),
                                                Vec::new(),
                                                |tree| {
                                                    tree.node_leaf(
                                                        block("large-logic-display"),
                                                        Vec::new(),
                                                    );
                                                    tree.node_leaf(
                                                        block("tile-logic-display"),
                                                        Vec::new(),
                                                    );
                                                },
                                            );

                                            tree.node_with_objectives(
                                                block("memory-cell"),
                                                Vec::new(),
                                                Vec::new(),
                                                |tree| {
                                                    tree.node_leaf(
                                                        block("memory-bank"),
                                                        Vec::new(),
                                                    );
                                                },
                                            );
                                        },
                                    );

                                    tree.node_with_objectives(
                                        block("logic-processor"),
                                        Vec::new(),
                                        Vec::new(),
                                        |tree| {
                                            tree.node_leaf(block("hyper-processor"), Vec::new());
                                        },
                                    );
                                },
                            );
                        },
                    );

                    tree.node_leaf(block("illuminator"), Vec::new());
                },
            );
        });

        tree.node_with_objectives(
            block("combustion-generator"),
            Vec::new(),
            vec![research(item("coal"))],
            |tree| {
                tree.node_with_objectives(block("power-node"), Vec::new(), Vec::new(), |tree| {
                    tree.node_with_objectives(
                        block("power-node-large"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_with_objectives(
                                block("diode"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_leaf(block("surge-tower"), Vec::new());
                                },
                            );
                        },
                    );

                    tree.node_with_objectives(block("battery"), Vec::new(), Vec::new(), |tree| {
                        tree.node_leaf(block("battery-large"), Vec::new());
                    });

                    tree.node_with_objectives(block("mender"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(
                            block("mend-projector"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_with_objectives(
                                    block("force-projector"),
                                    Vec::new(),
                                    vec![sector_complete("impact0078")],
                                    |tree| {
                                        tree.node_with_objectives(
                                            block("overdrive-projector"),
                                            Vec::new(),
                                            vec![sector_complete("impact0078")],
                                            |tree| {
                                                tree.node_with_objectives(
                                                    block("overdrive-dome"),
                                                    Vec::new(),
                                                    vec![sector_complete("desolateRift")],
                                                    |_| {},
                                                );
                                            },
                                        );
                                    },
                                );

                                tree.node_with_objectives(
                                    block("repair-point"),
                                    Vec::new(),
                                    Vec::new(),
                                    |tree| {
                                        tree.node_leaf(block("repair-turret"), Vec::new());
                                    },
                                );
                            },
                        );
                    });

                    tree.node_with_objectives(
                        block("steam-generator"),
                        Vec::new(),
                        vec![sector_complete("crateredBattleground")],
                        |tree| {
                            tree.node_with_objectives(
                                block("thermal-generator"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_with_objectives(
                                        block("differential-generator"),
                                        Vec::new(),
                                        Vec::new(),
                                        |tree| {
                                            tree.node_with_objectives(
                                                block("thorium-reactor"),
                                                Vec::new(),
                                                vec![
                                                    research(liquid("cryofluid")),
                                                    on_sector("nuclearComplex"),
                                                ],
                                                |tree| {
                                                    tree.node_leaf(
                                                        block("impact-reactor"),
                                                        Vec::new(),
                                                    );
                                                    tree.node_leaf(
                                                        block("rtg-generator"),
                                                        Vec::new(),
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
                        block("solar-panel"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_leaf(block("large-solar-panel"), Vec::new());
                        },
                    );
                });
            },
        );
    });
}

fn load_defense(tree: &mut TechTree) {
    tree.node_with_objectives(block("duo"), Vec::new(), Vec::new(), |tree| {
        tree.node_with_objectives(block("copper-wall"), Vec::new(), Vec::new(), |tree| {
            tree.node_with_objectives(block("copper-wall-large"), Vec::new(), Vec::new(), |tree| {
                tree.node_with_objectives(block("scrap-wall"), Vec::new(), Vec::new(), |tree| {
                    tree.node_with_objectives(
                        block("scrap-wall-large"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_with_objectives(
                                block("scrap-wall-huge"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_leaf(block("scrap-wall-gigantic"), Vec::new());
                                },
                            );
                        },
                    );
                });

                tree.node_with_objectives(block("titanium-wall"), Vec::new(), Vec::new(), |tree| {
                    tree.node_leaf(block("titanium-wall-large"), Vec::new());
                    tree.node_with_objectives(block("door"), Vec::new(), Vec::new(), |tree| {
                        tree.node_leaf(block("door-large"), Vec::new());
                    });
                    tree.node_with_objectives(
                        block("plastanium-wall"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_leaf(block("plastanium-wall-large"), Vec::new());
                        },
                    );
                    tree.node_with_objectives(
                        block("thorium-wall"),
                        Vec::new(),
                        Vec::new(),
                        |tree| {
                            tree.node_leaf(block("thorium-wall-large"), Vec::new());
                            tree.node_with_objectives(
                                block("surge-wall"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_leaf(block("surge-wall-large"), Vec::new());
                                    tree.node_with_objectives(
                                        block("phase-wall"),
                                        Vec::new(),
                                        Vec::new(),
                                        |tree| {
                                            tree.node_leaf(block("phase-wall-large"), Vec::new());
                                        },
                                    );
                                },
                            );
                        },
                    );
                });
            });
        });

        tree.node_with_objectives(block("scatter"), Vec::new(), Vec::new(), |tree| {
            tree.node_with_objectives(
                block("hail"),
                Vec::new(),
                vec![sector_complete("crateredBattleground")],
                |tree| {
                    tree.node_with_objectives(block("salvo"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(
                            block("swarmer"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_with_objectives(
                                    block("cyclone"),
                                    Vec::new(),
                                    Vec::new(),
                                    |tree| {
                                        tree.node_with_objectives(
                                            block("spectre"),
                                            Vec::new(),
                                            vec![sector_complete("nuclearComplex")],
                                            |_| {},
                                        );
                                    },
                                );
                            },
                        );

                        tree.node_with_objectives(
                            block("ripple"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_leaf(block("fuse"), Vec::new());
                            },
                        );
                    });
                },
            );
        });

        tree.node_with_objectives(
            block("arc"),
            Vec::new(),
            vec![on_sector("frozenForest")],
            |tree| {
                tree.node_with_objectives(block("scorch"), Vec::new(), Vec::new(), |tree| {
                    tree.node_with_objectives(block("wave"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(
                            block("parallax"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_leaf(block("segment"), Vec::new());
                            },
                        );

                        tree.node_with_objectives(
                            block("tsunami"),
                            Vec::new(),
                            vec![sector_complete("navalFortress")],
                            |_| {},
                        );
                    });

                    tree.node_with_objectives(block("lancer"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(
                            block("meltdown"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_leaf(block("foreshadow"), Vec::new());
                            },
                        );

                        tree.node_leaf(block("shock-mine"), Vec::new());
                    });
                });
            },
        );
    });
}

fn load_units(tree: &mut TechTree) {
    tree.node_with_objectives(block("ground-factory"), Vec::new(), Vec::new(), |tree| {
        tree.node_with_objectives(unit("dagger"), Vec::new(), Vec::new(), |tree| {
            tree.node_with_objectives(unit("mace"), Vec::new(), Vec::new(), |tree| {
                tree.node_with_objectives(unit("fortress"), Vec::new(), Vec::new(), |tree| {
                    tree.node_with_objectives(unit("scepter"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(
                            unit("reign"),
                            Vec::new(),
                            vec![sector_complete("desolateRift")],
                            |_| {},
                        );
                    });
                });
            });

            tree.node_with_objectives(
                unit("nova"),
                Vec::new(),
                vec![sector_complete("fungalPass")],
                |tree| {
                    tree.node_with_objectives(unit("pulsar"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(unit("quasar"), Vec::new(), Vec::new(), |tree| {
                            tree.node_with_objectives(
                                unit("vela"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_leaf(unit("corvus"), Vec::new());
                                },
                            );
                        });
                    });
                },
            );

            tree.node_with_objectives(
                unit("crawler"),
                vec![stack("silicon", 400), stack("graphite", 400)],
                Vec::new(),
                |tree| {
                    tree.node_with_objectives(unit("atrax"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(
                            unit("spiroct"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_with_objectives(
                                    unit("arkyid"),
                                    Vec::new(),
                                    Vec::new(),
                                    |tree| {
                                        tree.node_with_objectives(
                                            unit("toxopid"),
                                            Vec::new(),
                                            vec![sector_complete("mycelialBastion")],
                                            |_| {},
                                        );
                                    },
                                );
                            },
                        );
                    });
                },
            );
        });

        tree.node_with_objectives(block("air-factory"), Vec::new(), Vec::new(), |tree| {
            tree.node_with_objectives(unit("flare"), Vec::new(), Vec::new(), |tree| {
                tree.node_with_objectives(unit("horizon"), Vec::new(), Vec::new(), |tree| {
                    tree.node_with_objectives(unit("zenith"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(
                            unit("antumbra"),
                            Vec::new(),
                            Vec::new(),
                            |tree| {
                                tree.node_leaf(unit("eclipse"), Vec::new());
                            },
                        );
                    });
                });

                tree.node_with_objectives(unit("mono"), Vec::new(), Vec::new(), |tree| {
                    tree.node_with_objectives(unit("poly"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(unit("mega"), Vec::new(), Vec::new(), |tree| {
                            tree.node_with_objectives(
                                unit("quad"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_leaf(unit("oct"), Vec::new());
                                },
                            );
                        });
                    });
                });
            });

            tree.node_with_objectives(
                block("naval-factory"),
                Vec::new(),
                vec![sector_complete("ruinousShores")],
                |tree| {
                    tree.node_with_objectives(unit("risso"), Vec::new(), Vec::new(), |tree| {
                        tree.node_with_objectives(unit("minke"), Vec::new(), Vec::new(), |tree| {
                            tree.node_with_objectives(
                                unit("bryde"),
                                Vec::new(),
                                Vec::new(),
                                |tree| {
                                    tree.node_with_objectives(
                                        unit("sei"),
                                        Vec::new(),
                                        Vec::new(),
                                        |tree| {
                                            tree.node_with_objectives(
                                                unit("omura"),
                                                Vec::new(),
                                                vec![sector_complete("littoralShipyard")],
                                                |_| {},
                                            );
                                        },
                                    );
                                },
                            );
                        });

                        tree.node_with_objectives(
                            unit("retusa"),
                            Vec::new(),
                            vec![sector_complete("windsweptIslands")],
                            |tree| {
                                tree.node_with_objectives(
                                    unit("oxynoe"),
                                    Vec::new(),
                                    vec![sector_complete("coastline")],
                                    |tree| {
                                        tree.node_with_objectives(
                                            unit("cyerce"),
                                            Vec::new(),
                                            vec![sector_complete("perilousHarbor")],
                                            |tree| {
                                                tree.node_with_objectives(
                                                    unit("aegires"),
                                                    Vec::new(),
                                                    Vec::new(),
                                                    |tree| {
                                                        tree.node_with_objectives(
                                                            unit("navanax"),
                                                            Vec::new(),
                                                            vec![sector_complete("navalFortress")],
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
                    });
                },
            );
        });

        tree.node_with_objectives(
            block("additive-reconstructor"),
            Vec::new(),
            vec![sector_complete("fungalPass")],
            |tree| {
                tree.node_with_objectives(
                    block("multiplicative-reconstructor"),
                    Vec::new(),
                    vec![sector_complete("frontier")],
                    |tree| {
                        tree.node_with_objectives(
                            block("exponential-reconstructor"),
                            Vec::new(),
                            vec![sector_complete("overgrowth")],
                            |tree| {
                                tree.node_with_objectives(
                                    block("tetrative-reconstructor"),
                                    Vec::new(),
                                    vec![sector_complete("mycelialBastion")],
                                    |_| {},
                                );
                            },
                        );
                    },
                );
            },
        );
    });
}

fn load_sectors(tree: &mut TechTree) {
    tree.node_with_objectives(sector("groundZero"), Vec::new(), Vec::new(), |tree| {
        tree.node_with_objectives(
            sector("frozenForest"),
            Vec::new(),
            vec![
                sector_complete("groundZero"),
                research(block("junction")),
                research(block("router")),
            ],
            |tree| {
                tree.node_with_objectives(
                    sector("crateredBattleground"),
                    Vec::new(),
                    vec![
                        sector_complete("frozenForest"),
                        research(block("mender")),
                        research(block("combustion-generator")),
                    ],
                    |tree| {
                        load_mid_campaign_sectors(tree);
                    },
                );
            },
        );
    });
}

fn load_mid_campaign_sectors(tree: &mut TechTree) {
    tree.node_with_objectives(
        sector("ruinousShores"),
        Vec::new(),
        vec![
            sector_complete("crateredBattleground"),
            research(block("graphite-press")),
            research(block("kiln")),
            research(block("mechanical-pump")),
        ],
        |tree| {
            tree.node_with_objectives(
                sector("windsweptIslands"),
                Vec::new(),
                vec![
                    sector_complete("ruinousShores"),
                    research(block("pneumatic-drill")),
                    research(block("hail")),
                    research(block("silicon-smelter")),
                    research(block("steam-generator")),
                ],
                |tree| {
                    tree.node_with_objectives(
                        sector("saltFlats"),
                        Vec::new(),
                        vec![
                            sector_complete("windsweptIslands"),
                            sector_complete("fungalPass"),
                            sector_complete("frontier"),
                            research(block("ground-factory")),
                            research(block("additive-reconstructor")),
                            research(block("air-factory")),
                            research(block("door")),
                        ],
                        |tree| {
                            tree.node_with_objectives(
                                sector("tarFields"),
                                Vec::new(),
                                vec![
                                    sector_complete("saltFlats"),
                                    research(block("coal-centrifuge")),
                                    research(block("conduit")),
                                    research(block("wave")),
                                ],
                                |tree| {
                                    tree.node_with_objectives(
                                        sector("impact0078"),
                                        Vec::new(),
                                        vec![
                                            sector_complete("tarFields"),
                                            research(item("thorium")),
                                            research(block("lancer")),
                                            research(block("salvo")),
                                            research(block("core-foundation")),
                                        ],
                                        |tree| {
                                            tree.node_with_objectives(
                                                sector("desolateRift"),
                                                Vec::new(),
                                                vec![
                                                    sector_complete("impact0078"),
                                                    research(block("thermal-generator")),
                                                    research(block("thorium-reactor")),
                                                    research(block("core-nucleus")),
                                                ],
                                                |tree| {
                                                    tree.node_with_objectives(
                                                        sector("littoralShipyard"),
                                                        Vec::new(),
                                                        vec![
                                                            sector_complete("desolateRift"),
                                                            sector_complete("navalFortress"),
                                                            research(unit("risso")),
                                                            research(unit("minke")),
                                                            research(unit("bryde")),
                                                            research(unit("sei")),
                                                            research(block("spectre")),
                                                            research(block(
                                                                "additive-reconstructor",
                                                            )),
                                                            research(block(
                                                                "exponential-reconstructor",
                                                            )),
                                                        ],
                                                        |tree| {
                                                            tree.node_with_objectives(
                                                                sector("planetaryTerminal"),
                                                                Vec::new(),
                                                                vec![
                                                                    sector_complete(
                                                                        "nuclearComplex",
                                                                    ),
                                                                    sector_complete(
                                                                        "extractionOutpost",
                                                                    ),
                                                                    sector_complete(
                                                                        "mycelialBastion",
                                                                    ),
                                                                    sector_complete(
                                                                        "littoralShipyard",
                                                                    ),
                                                                    research(unit("omura")),
                                                                    research(block(
                                                                        "advanced-launch-pad",
                                                                    )),
                                                                    research(block("mass-driver")),
                                                                    research(block(
                                                                        "impact-reactor",
                                                                    )),
                                                                    research(block(
                                                                        "tetrative-reconstructor",
                                                                    )),
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

                            load_coastline_branch(tree);
                        },
                    );
                },
            );
        },
    );

    load_biomass_branch(tree);
}

fn load_coastline_branch(tree: &mut TechTree) {
    tree.node_with_objectives(
        sector("coastline"),
        Vec::new(),
        vec![
            sector_complete("tarFields"),
            sector_complete("saltFlats"),
            research(block("naval-factory")),
            research(block("payload-conveyor")),
        ],
        |tree| {
            tree.node_with_objectives(
                sector("testingGrounds"),
                Vec::new(),
                vec![
                    sector_complete("coastline"),
                    research(block("cryofluid-mixer")),
                    research(liquid("cryofluid")),
                    research(block("water-extractor")),
                    research(block("ripple")),
                ],
                |_| {},
            );

            tree.node_with_objectives(
                sector("navalFortress"),
                Vec::new(),
                vec![
                    sector_complete("coastline"),
                    sector_complete("extractionOutpost"),
                    research(block("core-nucleus")),
                    research(block("mass-driver")),
                    research(unit("oxynoe")),
                    research(unit("minke")),
                    research(unit("bryde")),
                    research(block("cyclone")),
                    research(block("ripple")),
                ],
                |tree| {
                    tree.node_with_objectives(
                        sector("sunkenPier"),
                        Vec::new(),
                        vec![
                            sector_complete("navalFortress"),
                            sector_complete("coastline"),
                            research(block("multiplicative-reconstructor")),
                        ],
                        |_| {},
                    );
                    tree.node_with_objectives(
                        sector("weatheredChannels"),
                        Vec::new(),
                        vec![
                            sector_complete("impact0078"),
                            sector_complete("navalFortress"),
                            research(unit("bryde")),
                            research(block("surge-smelter")),
                            research(block("overdrive-projector")),
                        ],
                        |_| {},
                    );
                },
            );
        },
    );
}

fn load_biomass_branch(tree: &mut TechTree) {
    tree.node_with_objectives(
        sector("biomassFacility"),
        Vec::new(),
        vec![
            sector_complete("crateredBattleground"),
            research(block("power-node")),
            research(block("steam-generator")),
            research(block("scatter")),
            research(block("graphite-press")),
        ],
        |tree| {
            tree.node_with_objectives(
                sector("stainedMountains"),
                Vec::new(),
                vec![
                    sector_complete("biomassFacility"),
                    research(block("pneumatic-drill")),
                    research(block("silicon-smelter")),
                ],
                |tree| {
                    tree.node_with_objectives(
                        sector("facility32m"),
                        Vec::new(),
                        vec![
                            research(block("plastanium-compressor")),
                            research(block("lancer")),
                            research(block("salvo")),
                            sector_complete("stainedMountains"),
                            sector_complete("windsweptIslands"),
                        ],
                        |_| {},
                    );

                    tree.node_with_objectives(
                        sector("infestedCanyons"),
                        Vec::new(),
                        vec![
                            sector_complete("fungalPass"),
                            sector_complete("frontier"),
                            research(block("naval-factory")),
                            research(unit("risso")),
                            research(unit("minke")),
                            research(block("additive-reconstructor")),
                        ],
                        |tree| {
                            tree.node_with_objectives(
                                sector("nuclearComplex"),
                                Vec::new(),
                                vec![
                                    sector_complete("infestedCanyons"),
                                    research(block("thermal-generator")),
                                    research(block("laser-drill")),
                                    research(item("plastanium")),
                                    research(block("swarmer")),
                                ],
                                |_| {},
                            );

                            tree.node_with_objectives(
                                sector("taintedWoods"),
                                Vec::new(),
                                vec![
                                    sector_complete("infestedCanyons"),
                                    research(item("spore-pod")),
                                    research(item("plastanium")),
                                    research(block("wave")),
                                ],
                                |_| {},
                            );
                        },
                    );
                },
            );

            tree.node_with_objectives(
                sector("fungalPass"),
                Vec::new(),
                vec![research(block("ground-factory")), research(unit("dagger"))],
                |tree| {
                    tree.node_with_objectives(
                        sector("frontier"),
                        Vec::new(),
                        vec![
                            sector_complete("biomassFacility"),
                            sector_complete("fungalPass"),
                            research(block("ground-factory")),
                            research(block("air-factory")),
                            research(block("additive-reconstructor")),
                            research(unit("mace")),
                            research(unit("mono")),
                        ],
                        |tree| {
                            load_frontier_children(tree);
                        },
                    );
                },
            );
        },
    );
}

fn load_frontier_children(tree: &mut TechTree) {
    tree.node_with_objectives(
        sector("perilousHarbor"),
        Vec::new(),
        vec![
            sector_complete("biomassFacility"),
            sector_complete("frontier"),
            research(block("naval-factory")),
            research(unit("risso")),
            research(unit("retusa")),
            research(block("steam-generator")),
            research(block("cultivator")),
            research(block("coal-centrifuge")),
        ],
        |tree| {
            tree.node_with_objectives(
                sector("extractionOutpost"),
                Vec::new(),
                vec![
                    sector_complete("windsweptIslands"),
                    sector_complete("perilousHarbor"),
                    sector_complete("facility32m"),
                    research(block("multiplicative-reconstructor")),
                    research(unit("risso")),
                    research(unit("minke")),
                    research(unit("fortress")),
                ],
                |tree| {
                    tree.node_with_objectives(
                        sector("atolls"),
                        Vec::new(),
                        vec![research(unit("poly")), research(unit("mega"))],
                        |_| {},
                    );
                },
            );
        },
    );

    tree.node_with_objectives(
        sector("overgrowth"),
        Vec::new(),
        vec![
            sector_complete("frontier"),
            sector_complete("windsweptIslands"),
            research(block("multiplicative-reconstructor")),
            research(unit("fortress")),
            research(block("ripple")),
            research(block("salvo")),
            research(block("cultivator")),
            research(block("spore-press")),
        ],
        |tree| {
            tree.node_with_objectives(
                sector("mycelialBastion"),
                Vec::new(),
                vec![
                    research(unit("atrax")),
                    research(unit("spiroct")),
                    research(unit("arkyid")),
                    research(block("multiplicative-reconstructor")),
                    research(block("exponential-reconstructor")),
                ],
                |_| {},
            );
        },
    );
}

fn load_produce_tree(tree: &mut TechTree) {
    tree.node_produce(item("copper"), Vec::new(), Vec::new(), |tree| {
        tree.node_produce(liquid("water"), Vec::new(), Vec::new(), |_| {});

        tree.node_produce(item("lead"), Vec::new(), Vec::new(), |tree| {
            tree.node_produce(item("titanium"), Vec::new(), Vec::new(), |tree| {
                tree.node_produce(liquid("cryofluid"), Vec::new(), Vec::new(), |_| {});
                tree.node_produce(item("thorium"), Vec::new(), Vec::new(), |tree| {
                    tree.node_produce(item("surge-alloy"), Vec::new(), Vec::new(), |_| {});
                    tree.node_produce(item("phase-fabric"), Vec::new(), Vec::new(), |_| {});
                });
            });

            tree.node_produce(item("metaglass"), Vec::new(), Vec::new(), |_| {});
        });

        tree.node_produce(item("sand"), Vec::new(), Vec::new(), |tree| {
            tree.node_produce(item("scrap"), Vec::new(), Vec::new(), |tree| {
                tree.node_produce(liquid("slag"), Vec::new(), Vec::new(), |_| {});
            });

            tree.node_produce(item("coal"), Vec::new(), Vec::new(), |tree| {
                tree.node_produce(item("graphite"), Vec::new(), Vec::new(), |tree| {
                    tree.node_produce(item("silicon"), Vec::new(), Vec::new(), |_| {});
                });

                tree.node_produce(item("pyratite"), Vec::new(), Vec::new(), |tree| {
                    tree.node_produce(item("blast-compound"), Vec::new(), Vec::new(), |_| {});
                });

                tree.node_produce(item("spore-pod"), Vec::new(), Vec::new(), |_| {});
                tree.node_produce(liquid("oil"), Vec::new(), Vec::new(), |tree| {
                    tree.node_produce(item("plastanium"), Vec::new(), Vec::new(), |_| {});
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
    fn serpulo_tree_root_and_key_distribution_branch_match_upstream() {
        let tree = load();
        let root = tree.node(tree.roots()[0]).unwrap();

        assert_eq!(root.name.as_deref(), Some("serpulo"));
        assert_eq!(root.content.name, "core-shard");
        assert_eq!(
            child_names(&tree, "core-shard"),
            vec![
                "conveyor",
                "core-foundation",
                "mechanical-drill",
                "duo",
                "ground-factory",
                "groundZero",
                "copper"
            ]
        );
        assert_eq!(child_names(&tree, "conveyor"), vec!["junction"]);
        assert_eq!(child_names(&tree, "junction"), vec!["router"]);
        assert_eq!(
            child_names(&tree, "router"),
            vec![
                "advanced-launch-pad",
                "distributor",
                "sorter",
                "container",
                "item-bridge"
            ]
        );
        assert_eq!(
            objective_strings(&tree, "advanced-launch-pad"),
            vec!["sectorComplete: extractionOutpost"]
        );
        assert_eq!(
            objective_strings(&tree, "interplanetary-accelerator"),
            vec!["sectorComplete: planetaryTerminal"]
        );
    }

    #[test]
    fn serpulo_production_and_power_key_paths_match_upstream() {
        let tree = load();

        assert_eq!(
            child_names(&tree, "mechanical-drill"),
            vec!["mechanical-pump", "graphite-press", "combustion-generator"]
        );
        assert_eq!(
            objective_strings(&tree, "pneumatic-drill"),
            vec!["sectorComplete: frozenForest"]
        );
        assert_eq!(
            objective_strings(&tree, "combustion-generator"),
            vec!["research: coal"]
        );
        assert_eq!(child_names(&tree, "power-node")[0], "power-node-large");
        assert_eq!(
            objective_strings(&tree, "thorium-reactor"),
            vec!["research: cryofluid", "onSector: nuclearComplex"]
        );
        assert_eq!(
            objective_strings(&tree, "kiln"),
            vec!["onSector: crateredBattleground"]
        );
    }

    #[test]
    fn serpulo_sector_branch_keeps_campaign_objectives_and_auto_parent_requirement() {
        let tree = load();

        assert_eq!(
            objective_strings(&tree, "frozenForest"),
            vec![
                "sectorComplete: groundZero",
                "research: junction",
                "research: router"
            ]
        );
        assert_eq!(
            objective_strings(&tree, "crateredBattleground"),
            vec![
                "sectorComplete: frozenForest",
                "research: mender",
                "research: combustion-generator"
            ]
        );
        assert_eq!(
            objective_strings(&tree, "ruinousShores"),
            vec![
                "sectorComplete: crateredBattleground",
                "research: graphite-press",
                "research: kiln",
                "research: mechanical-pump"
            ]
        );

        assert_eq!(
            objective_strings(&tree, "planetaryTerminal"),
            vec![
                "sectorComplete: nuclearComplex",
                "sectorComplete: extractionOutpost",
                "sectorComplete: mycelialBastion",
                "sectorComplete: littoralShipyard",
                "research: omura",
                "research: advanced-launch-pad",
                "research: mass-driver",
                "research: impact-reactor",
                "research: tetrative-reconstructor"
            ]
        );
    }

    #[test]
    fn serpulo_unit_and_produce_branches_keep_special_cases() {
        let tree = load();

        assert_eq!(
            node(&tree, "crawler").requirements,
            vec![stack("silicon", 400), stack("graphite", 400)]
        );
        assert_eq!(
            objective_strings(&tree, "reign"),
            vec!["sectorComplete: desolateRift"]
        );
        assert_eq!(
            objective_strings(&tree, "nova"),
            vec!["sectorComplete: fungalPass"]
        );

        assert!(node(&tree, "copper")
            .objectives
            .iter()
            .any(|objective| objective.java_to_string() == "produce: copper"));
        assert_eq!(child_names(&tree, "copper"), vec!["water", "lead", "sand"]);
        assert_eq!(child_names(&tree, "lead"), vec!["titanium", "metaglass"]);
        assert_eq!(
            child_names(&tree, "coal"),
            vec!["graphite", "pyratite", "spore-pod", "oil"]
        );
    }
}
