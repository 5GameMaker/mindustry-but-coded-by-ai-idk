/// Cache-layer registry mirrored from upstream `mindustry.graphics.CacheLayer`.
///
/// The Java side keeps a mutable runtime registry. In Rust we model the stable
/// set of cache-layer kinds as an enum and preserve the upstream order for the
/// built-in layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CacheLayer {
    #[default]
    None,
    Water,
    Mud,
    Tar,
    Slag,
    Arkycite,
    Cryofluid,
    Space,
    Normal,
    Walls,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheLayerTarget {
    FloorCache,
    EffectBuffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheLayerShaderHint {
    None,
    Liquid,
    Space,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheLayerBlendHint {
    Opaque,
    ShaderBlit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheLayerPassStep {
    BeginTarget,
    RenderTiles,
    EndTarget,
    BlitTarget,
    ResumeFloorDraw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheLayerInvalidationHint {
    Chunk,
    ChunkAndSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheLayerPassMetadata {
    pub target: CacheLayerTarget,
    pub blit_target: CacheLayerTarget,
    pub needs_fbo: bool,
    pub shader_hint: CacheLayerShaderHint,
    pub blend_hint: CacheLayerBlendHint,
    pub invalidation_hint: CacheLayerInvalidationHint,
    pub steps: &'static [CacheLayerPassStep],
}

impl CacheLayerPassMetadata {
    const DIRECT_STEPS: [CacheLayerPassStep; 1] = [CacheLayerPassStep::RenderTiles];
    const SHADER_STEPS: [CacheLayerPassStep; 5] = [
        CacheLayerPassStep::BeginTarget,
        CacheLayerPassStep::RenderTiles,
        CacheLayerPassStep::EndTarget,
        CacheLayerPassStep::BlitTarget,
        CacheLayerPassStep::ResumeFloorDraw,
    ];

    pub const fn direct() -> Self {
        Self {
            target: CacheLayerTarget::FloorCache,
            blit_target: CacheLayerTarget::FloorCache,
            needs_fbo: false,
            shader_hint: CacheLayerShaderHint::None,
            blend_hint: CacheLayerBlendHint::Opaque,
            invalidation_hint: CacheLayerInvalidationHint::Chunk,
            steps: &Self::DIRECT_STEPS,
        }
    }

    pub const fn shader(shader_hint: CacheLayerShaderHint) -> Self {
        Self {
            target: CacheLayerTarget::EffectBuffer,
            blit_target: CacheLayerTarget::FloorCache,
            needs_fbo: true,
            shader_hint,
            blend_hint: CacheLayerBlendHint::ShaderBlit,
            invalidation_hint: CacheLayerInvalidationHint::ChunkAndSettings,
            steps: &Self::SHADER_STEPS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheLayerEntry {
    pub id: usize,
    pub name: &'static str,
    pub layer: CacheLayer,
    pub liquid: bool,
    pub pass: CacheLayerPassMetadata,
}

impl CacheLayerEntry {
    pub const fn new(
        id: usize,
        name: &'static str,
        layer: CacheLayer,
        liquid: bool,
        pass: CacheLayerPassMetadata,
    ) -> Self {
        Self {
            id,
            name,
            layer,
            liquid,
            pass,
        }
    }

    pub const fn shader(
        id: usize,
        name: &'static str,
        layer: CacheLayer,
        liquid: bool,
        shader_hint: CacheLayerShaderHint,
    ) -> Self {
        Self::new(
            id,
            name,
            layer,
            liquid,
            CacheLayerPassMetadata::shader(shader_hint),
        )
    }

    pub const fn direct(id: usize, name: &'static str, layer: CacheLayer, liquid: bool) -> Self {
        Self::new(id, name, layer, liquid, CacheLayerPassMetadata::direct())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheLayerRegistryMetadata {
    pub entries: &'static [CacheLayerEntry; 9],
}

impl CacheLayerRegistryMetadata {
    pub const fn new(entries: &'static [CacheLayerEntry; 9]) -> Self {
        Self { entries }
    }

    pub fn entry(self, layer: CacheLayer) -> Option<&'static CacheLayerEntry> {
        self.entries.iter().find(|entry| entry.layer == layer)
    }

    pub fn by_name(self, name: &str) -> Option<&'static CacheLayerEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }
}

impl Default for CacheLayerRegistryMetadata {
    fn default() -> Self {
        Self::new(CacheLayer::builtin_entries())
    }
}

impl CacheLayer {
    pub const BUILTIN: [CacheLayerEntry; 9] = [
        CacheLayerEntry::shader(
            0,
            "water",
            CacheLayer::Water,
            true,
            CacheLayerShaderHint::Liquid,
        ),
        CacheLayerEntry::shader(
            1,
            "mud",
            CacheLayer::Mud,
            true,
            CacheLayerShaderHint::Liquid,
        ),
        CacheLayerEntry::shader(
            2,
            "tar",
            CacheLayer::Tar,
            true,
            CacheLayerShaderHint::Liquid,
        ),
        CacheLayerEntry::shader(
            3,
            "slag",
            CacheLayer::Slag,
            true,
            CacheLayerShaderHint::Liquid,
        ),
        CacheLayerEntry::shader(
            4,
            "arkycite",
            CacheLayer::Arkycite,
            true,
            CacheLayerShaderHint::Liquid,
        ),
        CacheLayerEntry::shader(
            5,
            "cryofluid",
            CacheLayer::Cryofluid,
            true,
            CacheLayerShaderHint::Liquid,
        ),
        CacheLayerEntry::shader(
            6,
            "space",
            CacheLayer::Space,
            false,
            CacheLayerShaderHint::Space,
        ),
        CacheLayerEntry::direct(7, "normal", CacheLayer::Normal, false),
        CacheLayerEntry::direct(8, "walls", CacheLayer::Walls, false),
    ];

    pub const fn is_liquid(self) -> bool {
        matches!(
            self,
            CacheLayer::Water
                | CacheLayer::Mud
                | CacheLayer::Tar
                | CacheLayer::Slag
                | CacheLayer::Arkycite
                | CacheLayer::Cryofluid
        )
    }

    pub const fn name(self) -> &'static str {
        match self {
            CacheLayer::None => "none",
            CacheLayer::Water => "water",
            CacheLayer::Mud => "mud",
            CacheLayer::Tar => "tar",
            CacheLayer::Slag => "slag",
            CacheLayer::Arkycite => "arkycite",
            CacheLayer::Cryofluid => "cryofluid",
            CacheLayer::Space => "space",
            CacheLayer::Normal => "normal",
            CacheLayer::Walls => "walls",
        }
    }

    pub const fn builtin_entries() -> &'static [CacheLayerEntry; 9] {
        &Self::BUILTIN
    }

    pub fn entry(self) -> Option<&'static CacheLayerEntry> {
        Self::builtin_entries()
            .iter()
            .find(|entry| entry.layer == self)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::builtin_entries()
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.layer)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CacheLayer, CacheLayerBlendHint, CacheLayerInvalidationHint, CacheLayerPassStep,
        CacheLayerShaderHint, CacheLayerTarget,
    };

    #[test]
    fn cache_layer_defaults_and_lookup_match_upstream_order() {
        assert_eq!(CacheLayer::default(), CacheLayer::None);
        assert!(!CacheLayer::None.is_liquid());
        assert!(CacheLayer::Water.is_liquid());
        assert_eq!(CacheLayer::Walls.name(), "walls");
        assert_eq!(
            CacheLayer::from_name("cryofluid"),
            Some(CacheLayer::Cryofluid)
        );
        assert_eq!(CacheLayer::from_name("missing"), None);
        assert_eq!(CacheLayer::Space.entry().map(|entry| entry.id), Some(6));
        assert_eq!(
            CacheLayer::Water.entry().map(|entry| entry.pass.needs_fbo),
            Some(true)
        );
        assert_eq!(
            CacheLayer::Walls.entry().map(|entry| entry.pass.needs_fbo),
            Some(false)
        );

        let expected = [
            "water",
            "mud",
            "tar",
            "slag",
            "arkycite",
            "cryofluid",
            "space",
            "normal",
            "walls",
        ];
        for (index, (entry, expected_name)) in CacheLayer::builtin_entries()
            .iter()
            .zip(expected)
            .enumerate()
        {
            assert_eq!(entry.id, index);
            assert_eq!(entry.name, expected_name);
            assert_eq!(entry.layer.name(), expected_name);
            assert_eq!(entry.liquid, entry.layer.is_liquid());
        }
    }

    #[test]
    fn cache_layer_metadata_captures_shader_and_blit_hints() {
        let water = CacheLayer::Water.entry().unwrap();
        assert_eq!(water.id, 0);
        assert_eq!(water.pass.target, CacheLayerTarget::EffectBuffer);
        assert_eq!(water.pass.blit_target, CacheLayerTarget::FloorCache);
        assert!(water.pass.needs_fbo);
        assert_eq!(water.pass.shader_hint, CacheLayerShaderHint::Liquid);
        assert_eq!(water.pass.blend_hint, CacheLayerBlendHint::ShaderBlit);
        assert_eq!(
            water.pass.steps,
            &[
                CacheLayerPassStep::BeginTarget,
                CacheLayerPassStep::RenderTiles,
                CacheLayerPassStep::EndTarget,
                CacheLayerPassStep::BlitTarget,
                CacheLayerPassStep::ResumeFloorDraw,
            ]
        );
        assert_eq!(
            water.pass.invalidation_hint,
            CacheLayerInvalidationHint::ChunkAndSettings
        );

        let slag = CacheLayer::Slag.entry().unwrap();
        assert_eq!(slag.id, 3);
        assert_eq!(slag.pass.shader_hint, CacheLayerShaderHint::Liquid);
        assert_eq!(slag.pass.steps, water.pass.steps);

        let space = CacheLayer::Space.entry().unwrap();
        assert_eq!(space.id, 6);
        assert!(!space.liquid);
        assert_eq!(space.pass.shader_hint, CacheLayerShaderHint::Space);
        assert_eq!(space.pass.target, CacheLayerTarget::EffectBuffer);
        assert_eq!(space.pass.blit_target, CacheLayerTarget::FloorCache);

        let walls = CacheLayer::Walls.entry().unwrap();
        assert_eq!(walls.id, 8);
        assert_eq!(walls.pass.target, CacheLayerTarget::FloorCache);
        assert_eq!(walls.pass.blit_target, CacheLayerTarget::FloorCache);
        assert!(!walls.pass.needs_fbo);
        assert_eq!(walls.pass.shader_hint, CacheLayerShaderHint::None);
        assert_eq!(walls.pass.blend_hint, CacheLayerBlendHint::Opaque);
        assert_eq!(walls.pass.steps, &[CacheLayerPassStep::RenderTiles]);
        assert_eq!(
            walls.pass.invalidation_hint,
            CacheLayerInvalidationHint::Chunk
        );
    }
}
