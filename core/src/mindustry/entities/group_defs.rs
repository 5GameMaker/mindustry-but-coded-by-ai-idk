#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityGroupKind {
    All,
    Player,
    Bullet,
    Unit,
    Build,
    Sync,
    Draw,
    Fire,
    Puddle,
    Weather,
    Label,
    PowerGraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroupDef {
    pub kind: EntityGroupKind,
    pub field_name: &'static str,
    pub component: &'static str,
    pub mapping: bool,
    pub spatial: bool,
    pub collide: bool,
}

impl GroupDef {
    pub const fn new(
        kind: EntityGroupKind,
        field_name: &'static str,
        component: &'static str,
    ) -> Self {
        Self {
            kind,
            field_name,
            component,
            mapping: false,
            spatial: false,
            collide: false,
        }
    }

    pub const fn mapping(mut self) -> Self {
        self.mapping = true;
        self
    }

    pub const fn spatial(mut self) -> Self {
        self.spatial = true;
        self
    }

    pub const fn collide(mut self) -> Self {
        self.collide = true;
        self
    }
}

pub const GROUP_DEFS: [GroupDef; 12] = [
    GroupDef::new(EntityGroupKind::All, "all", "Entityc"),
    GroupDef::new(EntityGroupKind::Player, "player", "Playerc").mapping(),
    GroupDef::new(EntityGroupKind::Bullet, "bullet", "Bulletc")
        .spatial()
        .collide(),
    GroupDef::new(EntityGroupKind::Unit, "unit", "Unitc")
        .spatial()
        .mapping(),
    GroupDef::new(EntityGroupKind::Build, "build", "Buildingc"),
    GroupDef::new(EntityGroupKind::Sync, "sync", "Syncc").mapping(),
    GroupDef::new(EntityGroupKind::Draw, "draw", "Drawc"),
    GroupDef::new(EntityGroupKind::Fire, "fire", "Firec"),
    GroupDef::new(EntityGroupKind::Puddle, "puddle", "Puddlec"),
    GroupDef::new(EntityGroupKind::Weather, "weather", "WeatherStatec"),
    GroupDef::new(EntityGroupKind::Label, "label", "WorldLabelc").mapping(),
    GroupDef::new(
        EntityGroupKind::PowerGraph,
        "powerGraph",
        "PowerGraphUpdaterc",
    ),
];

pub fn group_def(kind: EntityGroupKind) -> &'static GroupDef {
    GROUP_DEFS
        .iter()
        .find(|def| def.kind == kind)
        .expect("entity group definition is missing")
}

pub fn mapping_groups() -> impl Iterator<Item = &'static GroupDef> {
    GROUP_DEFS.iter().filter(|def| def.mapping)
}

pub fn spatial_groups() -> impl Iterator<Item = &'static GroupDef> {
    GROUP_DEFS.iter().filter(|def| def.spatial)
}

pub fn colliding_groups() -> impl Iterator<Item = &'static GroupDef> {
    GROUP_DEFS.iter().filter(|def| def.collide)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_defs_preserve_java_field_order_and_components() {
        let names = GROUP_DEFS
            .iter()
            .map(|def| (def.field_name, def.component))
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                ("all", "Entityc"),
                ("player", "Playerc"),
                ("bullet", "Bulletc"),
                ("unit", "Unitc"),
                ("build", "Buildingc"),
                ("sync", "Syncc"),
                ("draw", "Drawc"),
                ("fire", "Firec"),
                ("puddle", "Puddlec"),
                ("weather", "WeatherStatec"),
                ("label", "WorldLabelc"),
                ("powerGraph", "PowerGraphUpdaterc"),
            ]
        );
    }

    #[test]
    fn group_defs_preserve_mapping_spatial_and_collide_flags() {
        let mapping = mapping_groups()
            .map(|def| def.field_name)
            .collect::<Vec<_>>();
        assert_eq!(mapping, vec!["player", "unit", "sync", "label"]);

        let spatial = spatial_groups()
            .map(|def| def.field_name)
            .collect::<Vec<_>>();
        assert_eq!(spatial, vec!["bullet", "unit"]);

        let colliding = colliding_groups()
            .map(|def| def.field_name)
            .collect::<Vec<_>>();
        assert_eq!(colliding, vec!["bullet"]);
    }

    #[test]
    fn group_def_lookup_returns_expected_definition() {
        let bullet = group_def(EntityGroupKind::Bullet);

        assert_eq!(bullet.field_name, "bullet");
        assert!(bullet.spatial);
        assert!(bullet.collide);
        assert!(!bullet.mapping);
    }
}
