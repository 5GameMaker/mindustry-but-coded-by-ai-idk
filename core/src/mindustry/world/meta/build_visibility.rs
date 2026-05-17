#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildVisibility {
    Hidden,
    Shown,
    DebugOnly,
    EditorOnly,
    CoreZoneOnly,
    WorldProcessorOnly,
    SandboxOnly,
    CampaignOnly,
    LegacyLaunchPadOnly,
    NotLegacyLaunchPadOnly,
    LightingOnly,
    AmmoOnly,
    FogOnly,
}

impl BuildVisibility {
    pub const fn statically_visible(self) -> bool {
        matches!(self, BuildVisibility::Shown)
    }
}
