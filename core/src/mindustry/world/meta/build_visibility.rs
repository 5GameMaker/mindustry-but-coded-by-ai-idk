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
    pub const ALL: [BuildVisibility; 13] = [
        BuildVisibility::Hidden,
        BuildVisibility::Shown,
        BuildVisibility::DebugOnly,
        BuildVisibility::EditorOnly,
        BuildVisibility::CoreZoneOnly,
        BuildVisibility::WorldProcessorOnly,
        BuildVisibility::SandboxOnly,
        BuildVisibility::CampaignOnly,
        BuildVisibility::LegacyLaunchPadOnly,
        BuildVisibility::NotLegacyLaunchPadOnly,
        BuildVisibility::LightingOnly,
        BuildVisibility::AmmoOnly,
        BuildVisibility::FogOnly,
    ];

    pub fn ordinal(self) -> u8 {
        Self::ALL
            .iter()
            .position(|value| *value == self)
            .expect("BuildVisibility::ALL must contain every variant") as u8
    }

    pub const fn statically_visible(self) -> bool {
        matches!(self, BuildVisibility::Shown)
    }

    pub fn visible(self, context: BuildVisibilityContext) -> bool {
        match self {
            BuildVisibility::Hidden | BuildVisibility::DebugOnly => false,
            BuildVisibility::Shown => true,
            BuildVisibility::EditorOnly => context.editor,
            BuildVisibility::CoreZoneOnly => context.core_zone_present || !context.game,
            BuildVisibility::WorldProcessorOnly => {
                context.editor || context.allow_edit_world_processors
            }
            BuildVisibility::SandboxOnly => !context.has_state || context.infinite_resources,
            BuildVisibility::CampaignOnly => {
                !context.has_state || context.campaign || !context.game
            }
            BuildVisibility::LegacyLaunchPadOnly => {
                (!context.has_state || context.campaign && context.legacy_launch_pads)
                    && context.advanced_launch_pad_present
                    && context.advanced_launch_pad_unlocked
            }
            BuildVisibility::NotLegacyLaunchPadOnly => {
                !context.has_state
                    || !context.game
                    || context.infinite_resources
                    || context.campaign && !context.legacy_launch_pads
            }
            BuildVisibility::LightingOnly => {
                !context.has_state
                    || context.lighting
                    || context.campaign
                    || !context.game
                    || context.infinite_resources
            }
            BuildVisibility::AmmoOnly => !context.has_state || context.unit_ammo,
            BuildVisibility::FogOnly => {
                !context.has_state || context.fog || context.editor || !context.game
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildVisibilityContext {
    pub has_state: bool,
    pub game: bool,
    pub editor: bool,
    pub campaign: bool,
    pub infinite_resources: bool,
    pub core_zone_present: bool,
    pub allow_edit_world_processors: bool,
    pub legacy_launch_pads: bool,
    pub advanced_launch_pad_present: bool,
    pub advanced_launch_pad_unlocked: bool,
    pub lighting: bool,
    pub unit_ammo: bool,
    pub fog: bool,
}

impl Default for BuildVisibilityContext {
    fn default() -> Self {
        Self {
            has_state: true,
            game: true,
            editor: false,
            campaign: false,
            infinite_resources: false,
            core_zone_present: false,
            allow_edit_world_processors: false,
            legacy_launch_pads: false,
            advanced_launch_pad_present: false,
            advanced_launch_pad_unlocked: false,
            lighting: false,
            unit_ammo: false,
            fog: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildVisibility, BuildVisibilityContext};

    #[test]
    fn build_visibility_order_matches_java_static_fields() {
        assert_eq!(BuildVisibility::ALL.len(), 13);
        assert_eq!(BuildVisibility::Hidden.ordinal(), 0);
        assert_eq!(BuildVisibility::Shown.ordinal(), 1);
        assert_eq!(BuildVisibility::DebugOnly.ordinal(), 2);
        assert_eq!(BuildVisibility::EditorOnly.ordinal(), 3);
        assert_eq!(BuildVisibility::FogOnly.ordinal(), 12);
    }

    #[test]
    fn build_visibility_static_and_runtime_predicates_follow_java_rules() {
        let normal = BuildVisibilityContext::default();
        assert!(!BuildVisibility::Hidden.visible(normal));
        assert!(BuildVisibility::Shown.visible(normal));
        assert!(!BuildVisibility::DebugOnly.visible(normal));
        assert!(!BuildVisibility::Hidden.statically_visible());
        assert!(BuildVisibility::Shown.statically_visible());

        let editor = BuildVisibilityContext {
            editor: true,
            ..normal
        };
        assert!(BuildVisibility::EditorOnly.visible(editor));
        assert!(BuildVisibility::WorldProcessorOnly.visible(editor));
        assert!(BuildVisibility::FogOnly.visible(editor));

        let menu = BuildVisibilityContext {
            game: false,
            ..normal
        };
        assert!(BuildVisibility::CoreZoneOnly.visible(menu));
        assert!(BuildVisibility::CampaignOnly.visible(menu));
        assert!(BuildVisibility::NotLegacyLaunchPadOnly.visible(menu));
        assert!(BuildVisibility::LightingOnly.visible(menu));
        assert!(BuildVisibility::FogOnly.visible(menu));

        let sandbox = BuildVisibilityContext {
            infinite_resources: true,
            ..normal
        };
        assert!(BuildVisibility::SandboxOnly.visible(sandbox));
        assert!(BuildVisibility::NotLegacyLaunchPadOnly.visible(sandbox));

        let campaign_legacy = BuildVisibilityContext {
            campaign: true,
            legacy_launch_pads: true,
            advanced_launch_pad_present: true,
            advanced_launch_pad_unlocked: true,
            ..normal
        };
        assert!(BuildVisibility::CampaignOnly.visible(campaign_legacy));
        assert!(BuildVisibility::LegacyLaunchPadOnly.visible(campaign_legacy));
        assert!(!BuildVisibility::NotLegacyLaunchPadOnly.visible(campaign_legacy));

        assert!(BuildVisibility::AmmoOnly.visible(BuildVisibilityContext {
            unit_ammo: true,
            ..normal
        }));
        assert!(
            BuildVisibility::LightingOnly.visible(BuildVisibilityContext {
                lighting: true,
                ..normal
            })
        );
        assert!(BuildVisibility::FogOnly.visible(BuildVisibilityContext {
            fog: true,
            ..normal
        }));
    }

    #[test]
    fn build_visibility_null_state_branches_are_visible_like_java() {
        let no_state = BuildVisibilityContext {
            has_state: false,
            ..BuildVisibilityContext::default()
        };
        assert!(BuildVisibility::SandboxOnly.visible(no_state));
        assert!(BuildVisibility::CampaignOnly.visible(no_state));
        assert!(BuildVisibility::NotLegacyLaunchPadOnly.visible(no_state));
        assert!(BuildVisibility::LightingOnly.visible(no_state));
        assert!(BuildVisibility::AmmoOnly.visible(no_state));
        assert!(BuildVisibility::FogOnly.visible(no_state));
    }
}
