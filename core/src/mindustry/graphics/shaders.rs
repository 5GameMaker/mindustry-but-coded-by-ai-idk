//! Backend-neutral shader registry and lifecycle plans mirrored from upstream
//! `mindustry.graphics.Shaders`.
//!
//! The Java implementation constructs concrete `Shader`/`Texture` instances in
//! `Shaders.init()` and writes uniforms directly from each shader's `apply()`
//! method. This Rust port keeps the same built-in shader names, source files,
//! default parameters and apply-time uniform semantics as data. A renderer can
//! consume the returned plans and map them onto any GPU backend.

/// Fixed-size RGBA color used by shader parameters and uniforms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ShaderColor {
    pub const CLEAR: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const LIGHT_AMBIENT: Self = Self::new(0.01, 0.01, 0.04, 0.99);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    pub const fn as_vec4(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub const fn as_vec3(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl Default for ShaderColor {
    fn default() -> Self {
        Self::CLEAR
    }
}

/// View/camera dimensions needed by screenspace shaders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderViewport {
    pub width: f32,
    pub height: f32,
}

impl ShaderViewport {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn valid(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }

    pub fn inverse(self) -> Option<[f32; 2]> {
        self.valid().then(|| [1.0 / self.width, 1.0 / self.height])
    }
}

/// Camera snapshot consumed by shader apply plans.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderCamera {
    pub position: [f32; 3],
    pub viewport: ShaderViewport,
    pub direction: [f32; 3],
}

impl ShaderCamera {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            position: [x, y, 0.0],
            viewport: ShaderViewport::new(width, height),
            direction: [0.0, 0.0, -1.0],
        }
    }

    pub fn lower_left(self) -> [f32; 2] {
        [
            self.position[0] - self.viewport.width / 2.0,
            self.position[1] - self.viewport.height / 2.0,
        ]
    }
}

/// Texture-region metadata required by build shaders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderTextureRegion {
    pub u: f32,
    pub v: f32,
    pub u2: f32,
    pub v2: f32,
    pub texture_size: ShaderViewport,
}

impl ShaderTextureRegion {
    pub const fn new(u: f32, v: f32, u2: f32, v2: f32, width: f32, height: f32) -> Self {
        Self {
            u,
            v,
            u2,
            v2,
            texture_size: ShaderViewport::new(width, height),
        }
    }

    /// Matches `BlockBuildShader`'s upstream fallback for an empty region.
    pub const fn blockbuild_fallback() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0, 1.0, 1.0)
    }
}

/// Planet information required by the planet/atmosphere plan layer.
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderPlanetState {
    pub position: [f32; 3],
    pub rotation: f32,
    pub radius: f32,
    pub atmosphere_rad_in: f32,
    pub atmosphere_rad_out: f32,
    pub atmosphere_color: ShaderColor,
    pub light_normal: [f32; 3],
    /// Symbolic matrix identifiers are intentionally backend-neutral. A renderer
    /// may resolve these labels to concrete matrices before upload.
    pub model_matrix: String,
    pub projection_matrix: String,
    pub inverse_projection_matrix: String,
}

impl Default for ShaderPlanetState {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: 0.0,
            radius: 1.0,
            atmosphere_rad_in: 0.0,
            atmosphere_rad_out: 0.0,
            atmosphere_color: ShaderColor::WHITE,
            light_normal: normalized_one(),
            model_matrix: "planet.model".to_string(),
            projection_matrix: "camera.combined".to_string(),
            inverse_projection_matrix: "camera.invProjectionView".to_string(),
        }
    }
}

/// Shockwave entry equivalent to upstream `(x, y, radius, life, lifetime)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShockwaveEntry {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    /// Remaining life, matching Java's 1 -> 0 convention.
    pub life: f32,
    pub lifetime: f32,
}

impl ShockwaveEntry {
    pub const fn new(x: f32, y: f32, radius: f32, lifetime: f32) -> Self {
        Self {
            x,
            y,
            radius,
            life: 1.0,
            lifetime,
        }
    }

    pub fn uniform_row(self) -> [f32; 4] {
        [self.x, self.y, self.radius * (1.0 - self.life), self.life]
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        if self.lifetime <= 0.0 {
            self.life = 0.0;
        } else {
            self.life -= delta / self.lifetime;
        }
        self.life > 0.0
    }
}

/// Mutable shockwave data model. It mirrors upstream max/size/lifetime behavior
/// without owning framebuffers or issuing draw calls.
#[derive(Debug, Clone, PartialEq)]
pub struct ShockwaveState {
    pub entries: Vec<ShockwaveEntry>,
    pub default_lifetime: f32,
}

impl ShockwaveState {
    pub const MAX: usize = 64;
    pub const DEFAULT_LIFETIME: f32 = 20.0;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, x: f32, y: f32, radius: f32) {
        self.add_with_lifetime(x, y, radius, self.default_lifetime);
    }

    pub fn add_with_lifetime(&mut self, x: f32, y: f32, radius: f32, lifetime: f32) {
        let entry = ShockwaveEntry::new(x, y, radius, lifetime);
        if self.entries.len() >= Self::MAX {
            self.entries[0] = entry;
        } else {
            self.entries.push(entry);
        }
    }

    pub fn update(&mut self, delta: f32, paused: bool, menu: bool) {
        if paused {
            return;
        }
        if menu {
            self.entries.clear();
            return;
        }
        self.entries.retain_mut(|entry| entry.tick(delta));
    }

    pub fn uniforms(&self) -> Vec<[f32; 4]> {
        self.entries
            .iter()
            .map(|entry| entry.uniform_row())
            .collect()
    }
}

impl Default for ShockwaveState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            default_lifetime: Self::DEFAULT_LIFETIME,
        }
    }
}

/// Runtime values that Java stores as fields on individual shader subclasses.
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderParameters {
    pub progress: f32,
    pub time: f32,
    pub alpha: f32,
    pub color: ShaderColor,
    pub region: Option<ShaderTextureRegion>,

    pub ambient: ShaderColor,
    pub light_dir: [f32; 3],
    pub ambient_color: ShaderColor,
    pub cam_dir: [f32; 3],
    pub cam_pos: [f32; 3],
    pub emissive: bool,
    pub cloud_alpha: f32,
    pub mouse: [f32; 3],
    pub planet: Option<ShaderPlanetState>,

    pub shockwaves: ShockwaveState,
}

impl Default for ShaderParameters {
    fn default() -> Self {
        Self {
            progress: 0.0,
            time: 0.0,
            alpha: 1.0,
            color: ShaderColor::CLEAR,
            region: None,

            ambient: ShaderColor::LIGHT_AMBIENT,
            light_dir: normalized_one(),
            ambient_color: ShaderColor::WHITE,
            cam_dir: [0.0, 0.0, 0.0],
            cam_pos: [0.0, 0.0, 0.0],
            emissive: false,
            cloud_alpha: 1.0,
            mouse: [0.0, 0.0, 0.0],
            planet: None,

            shockwaves: ShockwaveState::default(),
        }
    }
}

/// Apply-time environment that Java reads from `Core`, `Vars.renderer` and
/// `Time`.
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderApplyContext {
    pub camera: Option<ShaderCamera>,
    pub graphics: Option<ShaderViewport>,
    pub time: f32,
    pub global_time: f32,
    /// Equivalent to `Scl.scl(1f)` for screenspace effects.
    pub dp: f32,
    pub effect_buffer_texture: Option<String>,
    pub noise_texture: Option<String>,
    pub stars_texture: Option<String>,
    pub parameters: ShaderParameters,
}

impl Default for ShaderApplyContext {
    fn default() -> Self {
        Self {
            camera: None,
            graphics: None,
            time: 0.0,
            global_time: 0.0,
            dp: 1.0,
            effect_buffer_texture: Some("renderer.effectBuffer.texture".to_string()),
            noise_texture: Some("sprites/noise.png".to_string()),
            stars_texture: Some("sprites/space.png".to_string()),
            parameters: ShaderParameters::default(),
        }
    }
}

/// Upstream shader singleton names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderId {
    Mesh,
    BlockBuild,
    Shield,
    Fog,
    BuildBeam,
    Build,
    Armor,
    Darkness,
    Light,
    Water,
    Arkycite,
    Mud,
    Tar,
    Slag,
    Cryofluid,
    Space,
    Caustics,
    Planet,
    Clouds,
    PlanetGrid,
    Atmosphere,
    Unlit,
    UnlitWhite,
    Screenspace,
    Shockwave,
}

impl ShaderId {
    /// Built-ins constructed by upstream `Shaders.init()`, preserving order.
    pub const INIT_ORDER: [Self; 24] = [
        Self::Mesh,
        Self::BlockBuild,
        Self::Shield,
        Self::Fog,
        Self::BuildBeam,
        Self::Build,
        Self::Armor,
        Self::Darkness,
        Self::Light,
        Self::Water,
        Self::Arkycite,
        Self::Mud,
        Self::Tar,
        Self::Slag,
        Self::Cryofluid,
        Self::Space,
        Self::Caustics,
        Self::Planet,
        Self::Clouds,
        Self::PlanetGrid,
        Self::Atmosphere,
        Self::Unlit,
        Self::UnlitWhite,
        Self::Screenspace,
    ];

    /// All modeled shaders. `Shockwave` is included but disabled in upstream
    /// `init()` at v157.4.
    pub const ALL: [Self; 25] = [
        Self::Mesh,
        Self::BlockBuild,
        Self::Shield,
        Self::Fog,
        Self::BuildBeam,
        Self::Build,
        Self::Armor,
        Self::Darkness,
        Self::Light,
        Self::Water,
        Self::Arkycite,
        Self::Mud,
        Self::Tar,
        Self::Slag,
        Self::Cryofluid,
        Self::Space,
        Self::Caustics,
        Self::Planet,
        Self::Clouds,
        Self::PlanetGrid,
        Self::Atmosphere,
        Self::Unlit,
        Self::UnlitWhite,
        Self::Screenspace,
        Self::Shockwave,
    ];

    pub const fn field_name(self) -> &'static str {
        match self {
            Self::Mesh => "mesh",
            Self::BlockBuild => "blockbuild",
            Self::Shield => "shield",
            Self::Fog => "fog",
            Self::BuildBeam => "buildBeam",
            Self::Build => "build",
            Self::Armor => "armor",
            Self::Darkness => "darkness",
            Self::Light => "light",
            Self::Water => "water",
            Self::Arkycite => "arkycite",
            Self::Mud => "mud",
            Self::Tar => "tar",
            Self::Slag => "slag",
            Self::Cryofluid => "cryofluid",
            Self::Space => "space",
            Self::Caustics => "caustics",
            Self::Planet => "planet",
            Self::Clouds => "clouds",
            Self::PlanetGrid => "planetGrid",
            Self::Atmosphere => "atmosphere",
            Self::Unlit => "unlit",
            Self::UnlitWhite => "unlitWhite",
            Self::Screenspace => "screenspace",
            Self::Shockwave => "shockwave",
        }
    }

    pub const fn source(self) -> ShaderSource {
        match self {
            Self::Mesh => ShaderSource::new("planet.vert", "mesh.frag"),
            Self::BlockBuild => ShaderSource::new("default.vert", "blockbuild.frag"),
            Self::Shield => ShaderSource::new("screenspace.vert", "shield.frag"),
            Self::Fog => ShaderSource::new("default.vert", "fog.frag"),
            Self::BuildBeam => ShaderSource::new("screenspace.vert", "buildbeam.frag"),
            Self::Build => ShaderSource::new("default.vert", "unitbuild.frag"),
            Self::Armor => ShaderSource::new("default.vert", "unitarmor.frag"),
            Self::Darkness => ShaderSource::new("default.vert", "darkness.frag"),
            Self::Light => ShaderSource::new("screenspace.vert", "light.frag"),
            Self::Water => ShaderSource::new("screenspace.vert", "water.frag"),
            Self::Arkycite => ShaderSource::new("screenspace.vert", "arkycite.frag"),
            Self::Mud => ShaderSource::new("screenspace.vert", "mud.frag"),
            Self::Tar => ShaderSource::new("screenspace.vert", "tar.frag"),
            Self::Slag => ShaderSource::new("screenspace.vert", "slag.frag"),
            Self::Cryofluid => ShaderSource::new("screenspace.vert", "cryofluid.frag"),
            Self::Space => ShaderSource::new("screenspace.vert", "space.frag"),
            Self::Caustics => ShaderSource::new("screenspace.vert", "caustics.frag"),
            Self::Planet => ShaderSource::new("planet.vert", "planet.frag"),
            Self::Clouds => ShaderSource::new("planet.vert", "clouds.frag"),
            Self::PlanetGrid => ShaderSource::new("planetgrid.vert", "planetgrid.frag"),
            Self::Atmosphere => ShaderSource::new("atmosphere.vert", "atmosphere.frag"),
            // `LoadShader("planet", "unlit")`: vertex first after expansion.
            Self::Unlit => ShaderSource::new("unlit.vert", "planet.frag"),
            Self::UnlitWhite => ShaderSource::new("unlitwhite.vert", "planet.frag"),
            Self::Screenspace => ShaderSource::new("screenspace.vert", "screenspace.frag"),
            Self::Shockwave => ShaderSource::new("screenspace.vert", "shockwave.frag"),
        }
    }

    pub const fn class_name(self) -> &'static str {
        match self {
            Self::Mesh => "MeshShader",
            Self::BlockBuild => "BlockBuildShader",
            Self::Shield => "ShieldShader",
            Self::Fog => "FogShader",
            Self::BuildBeam => "BuildBeamShader",
            Self::Build => "UnitBuildShader",
            Self::Armor => "UnitArmorShader",
            Self::Darkness => "DarknessShader",
            Self::Light => "LightShader",
            Self::Water | Self::Arkycite | Self::Mud | Self::Tar | Self::Slag | Self::Cryofluid => {
                "SurfaceShader"
            }
            Self::Space => "SpaceShader",
            Self::Caustics => "SurfaceShader(caustics texture)",
            Self::Planet => "PlanetShader",
            Self::Clouds => "CloudShader",
            Self::PlanetGrid => "PlanetGridShader",
            Self::Atmosphere => "AtmosphereShader",
            Self::Unlit | Self::UnlitWhite | Self::Screenspace => "LoadShader",
            Self::Shockwave => "ShockwaveShader",
        }
    }

    pub const fn loaded_by_default(self) -> bool {
        !matches!(self, Self::Shockwave)
    }

    pub const fn optional_load(self) -> bool {
        matches!(self, Self::Shield)
    }

    pub const fn texture_name(self) -> Option<&'static str> {
        match self {
            Self::Caustics => Some("caustics"),
            Self::Water
            | Self::Arkycite
            | Self::Mud
            | Self::Tar
            | Self::Slag
            | Self::Cryofluid
            | Self::Space => Some("noise"),
            _ => None,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let canonical = canonical_name(name);
        Self::ALL
            .iter()
            .copied()
            .find(|id| canonical_name(id.field_name()) == canonical)
            .or_else(|| match canonical.as_str() {
                "unitbuild" => Some(Self::Build),
                "unitarmor" => Some(Self::Armor),
                "buildbeam" => Some(Self::BuildBeam),
                "planetgrid" => Some(Self::PlanetGrid),
                "unlitwhite" => Some(Self::UnlitWhite),
                _ => None,
            })
    }

    pub fn default_uniforms(self) -> Vec<UniformBinding> {
        let params = ShaderParameters::default();
        default_uniforms_for(self, &params)
    }
}

/// Shader source files under upstream `core/assets/shaders`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShaderSource {
    pub vertex: &'static str,
    pub fragment: &'static str,
}

impl ShaderSource {
    pub const fn new(vertex: &'static str, fragment: &'static str) -> Self {
        Self { vertex, fragment }
    }

    pub fn vertex_path(self) -> String {
        format!("shaders/{}", self.vertex)
    }

    pub fn fragment_path(self) -> String {
        format!("shaders/{}", self.fragment)
    }
}

/// Texture load requests needed by shader construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderTextureRequest {
    pub path: String,
    pub role: TextureRole,
    pub mag_filter: TextureFilter,
    pub min_filter: TextureFilter,
    pub wrap_u: TextureWrap,
    pub wrap_v: TextureWrap,
    pub gen_mip_maps: bool,
}

impl ShaderTextureRequest {
    pub fn noise(texture_name: &str) -> Self {
        Self {
            path: format!("sprites/{}.png", texture_name),
            role: TextureRole::Noise,
            mag_filter: TextureFilter::Linear,
            min_filter: TextureFilter::Linear,
            wrap_u: TextureWrap::Repeat,
            wrap_v: TextureWrap::Repeat,
            gen_mip_maps: false,
        }
    }

    pub fn stars() -> Self {
        Self {
            path: "sprites/space.png".to_string(),
            role: TextureRole::Stars,
            mag_filter: TextureFilter::Linear,
            min_filter: TextureFilter::MipMapLinearLinear,
            wrap_u: TextureWrap::MirroredRepeat,
            wrap_v: TextureWrap::MirroredRepeat,
            gen_mip_maps: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureRole {
    Noise,
    Stars,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFilter {
    Linear,
    MipMapLinearLinear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureWrap {
    Repeat,
    MirroredRepeat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderDescriptor {
    pub id: ShaderId,
    pub field_name: &'static str,
    pub class_name: &'static str,
    pub source: ShaderSource,
    pub loaded_by_default: bool,
    pub optional_load: bool,
}

impl ShaderDescriptor {
    pub fn texture_requests(&self) -> Vec<ShaderTextureRequest> {
        texture_requests_for(self.id)
    }
}

impl From<ShaderId> for ShaderDescriptor {
    fn from(id: ShaderId) -> Self {
        Self {
            id,
            field_name: id.field_name(),
            class_name: id.class_name(),
            source: id.source(),
            loaded_by_default: id.loaded_by_default(),
            optional_load: id.optional_load(),
        }
    }
}

/// Uniform value descriptions. Matrix and texture values are symbolic so that
/// this module remains renderer-agnostic.
#[derive(Debug, Clone, PartialEq)]
pub enum UniformValue {
    Int(i32),
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat4(String),
    Vec4Array(Vec<[f32; 4]>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UniformBinding {
    pub name: &'static str,
    pub value: UniformValue,
}

impl UniformBinding {
    pub fn new(name: &'static str, value: UniformValue) -> Self {
        Self { name, value }
    }
}

/// Backend-neutral operation list for one `apply()` call.
#[derive(Debug, Clone, PartialEq)]
pub enum ShaderApplyOperation {
    SetUniform(UniformBinding),
    SetUniformIfPresent(UniformBinding),
    BindTexture {
        uniform: &'static str,
        slot: u8,
        texture: TextureBinding,
    },
    ShockwavePreDraw {
        buffer: &'static str,
        clear_color: ShaderColor,
    },
    ShockwavePostDraw {
        buffer: &'static str,
        blend_disabled: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureBinding {
    Asset(String),
    EffectBuffer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShaderApplyPlan {
    pub shader: ShaderId,
    pub operations: Vec<ShaderApplyOperation>,
    pub errors: Vec<ShaderPlanError>,
}

impl ShaderApplyPlan {
    pub fn new(shader: ShaderId) -> Self {
        Self {
            shader,
            operations: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn uniforms(&self) -> impl Iterator<Item = &UniformBinding> {
        self.operations
            .iter()
            .filter_map(|operation| match operation {
                ShaderApplyOperation::SetUniform(binding)
                | ShaderApplyOperation::SetUniformIfPresent(binding) => Some(binding),
                _ => None,
            })
    }

    fn set(&mut self, name: &'static str, value: UniformValue) {
        self.operations
            .push(ShaderApplyOperation::SetUniform(UniformBinding::new(
                name, value,
            )));
    }

    fn set_if_present(&mut self, name: &'static str, value: UniformValue) {
        self.operations
            .push(ShaderApplyOperation::SetUniformIfPresent(
                UniformBinding::new(name, value),
            ));
    }

    fn bind(&mut self, uniform: &'static str, slot: u8, texture: TextureBinding) {
        self.operations.push(ShaderApplyOperation::BindTexture {
            uniform,
            slot,
            texture,
        });
    }

    fn error(&mut self, kind: ShaderPlanErrorKind, detail: impl Into<String>) {
        self.errors.push(ShaderPlanError {
            shader: Some(self.shader),
            kind,
            detail: detail.into(),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderPlanError {
    pub shader: Option<ShaderId>,
    pub kind: ShaderPlanErrorKind,
    pub detail: String,
}

impl ShaderPlanError {
    pub fn global(kind: ShaderPlanErrorKind, detail: impl Into<String>) -> Self {
        Self {
            shader: None,
            kind,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderPlanErrorKind {
    DuplicateName,
    EmptySourcePath,
    MissingCamera,
    MissingGraphics,
    MissingRegion,
    MissingPlanet,
    MissingTexture,
    InvalidViewport,
    InvalidScale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderLoadTask {
    pub shader: ShaderId,
    pub source: ShaderSource,
    pub optional: bool,
    pub enabled: bool,
    pub texture_requests: Vec<ShaderTextureRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderLoadPlan {
    pub tasks: Vec<ShaderLoadTask>,
    pub errors: Vec<ShaderPlanError>,
}

impl ShaderLoadPlan {
    pub fn enabled_tasks(&self) -> impl Iterator<Item = &ShaderLoadTask> {
        self.tasks.iter().filter(|task| task.enabled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShaderReloadAction {
    DropCachedProgram(ShaderId),
    Recreate(ShaderLoadTask),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderReloadPlan {
    pub actions: Vec<ShaderReloadAction>,
    pub errors: Vec<ShaderPlanError>,
}

/// Registry facade for built-in shader descriptors and lifecycle plans.
#[derive(Debug, Clone, Default)]
pub struct ShaderCatalog;

impl ShaderCatalog {
    pub fn descriptors() -> Vec<ShaderDescriptor> {
        ShaderId::ALL
            .iter()
            .copied()
            .map(ShaderDescriptor::from)
            .collect()
    }

    pub fn descriptor(id: ShaderId) -> ShaderDescriptor {
        ShaderDescriptor::from(id)
    }

    pub fn init_plan() -> ShaderLoadPlan {
        let mut plan = ShaderLoadPlan {
            tasks: ShaderId::ALL
                .iter()
                .copied()
                .map(|shader| ShaderLoadTask {
                    shader,
                    source: shader.source(),
                    optional: shader.optional_load(),
                    enabled: shader.loaded_by_default(),
                    texture_requests: texture_requests_for(shader),
                })
                .collect(),
            errors: Vec::new(),
        };
        plan.errors.extend(Self::validate().into_iter());
        plan
    }

    pub fn reload_plan() -> ShaderReloadPlan {
        let init = Self::init_plan();
        let mut actions = Vec::with_capacity(init.tasks.len() * 2);
        for task in init.tasks.into_iter().filter(|task| task.enabled) {
            actions.push(ShaderReloadAction::DropCachedProgram(task.shader));
            actions.push(ShaderReloadAction::Recreate(task));
        }
        ShaderReloadPlan {
            actions,
            errors: init.errors,
        }
    }

    pub fn apply_plan(shader: ShaderId, context: &ShaderApplyContext) -> ShaderApplyPlan {
        apply_plan_for(shader, context)
    }

    pub fn validate() -> Vec<ShaderPlanError> {
        let mut errors = Vec::new();
        let all = ShaderId::ALL;

        for shader in all {
            let source = shader.source();
            if source.vertex.is_empty() || source.fragment.is_empty() {
                errors.push(ShaderPlanError {
                    shader: Some(shader),
                    kind: ShaderPlanErrorKind::EmptySourcePath,
                    detail: "shader source path is empty".to_string(),
                });
            }
        }

        for (index, shader) in all.iter().enumerate() {
            for other in all.iter().skip(index + 1) {
                if shader.field_name() == other.field_name() {
                    errors.push(ShaderPlanError::global(
                        ShaderPlanErrorKind::DuplicateName,
                        format!("duplicate shader field name '{}'", shader.field_name()),
                    ));
                }
            }
        }

        errors
    }
}

fn apply_plan_for(shader: ShaderId, context: &ShaderApplyContext) -> ShaderApplyPlan {
    let mut plan = ShaderApplyPlan::new(shader);
    let params = &context.parameters;

    match shader {
        ShaderId::Mesh
        | ShaderId::Darkness
        | ShaderId::Fog
        | ShaderId::Unlit
        | ShaderId::UnlitWhite
        | ShaderId::Screenspace => {}
        ShaderId::BlockBuild => apply_blockbuild(&mut plan, params),
        ShaderId::Shield | ShaderId::BuildBeam => apply_screenspace_effect(&mut plan, context),
        ShaderId::Build => apply_unit_build(&mut plan, params),
        ShaderId::Armor => apply_unit_armor(&mut plan, params),
        ShaderId::Light => {
            plan.set("u_ambient", UniformValue::Vec4(params.ambient.as_vec4()));
        }
        ShaderId::Water
        | ShaderId::Arkycite
        | ShaderId::Mud
        | ShaderId::Tar
        | ShaderId::Slag
        | ShaderId::Cryofluid
        | ShaderId::Caustics => apply_surface(&mut plan, context, false),
        ShaderId::Space => apply_surface(&mut plan, context, true),
        ShaderId::Planet => apply_planet(&mut plan, context),
        ShaderId::Clouds => apply_clouds(&mut plan, context),
        ShaderId::PlanetGrid => {
            plan.set("u_mouse", UniformValue::Vec3(params.mouse));
        }
        ShaderId::Atmosphere => apply_atmosphere(&mut plan, context),
        ShaderId::Shockwave => apply_shockwave(&mut plan, context),
    }

    plan
}

fn apply_blockbuild(plan: &mut ShaderApplyPlan, params: &ShaderParameters) {
    plan.set("u_progress", UniformValue::Float(params.progress));
    plan.set("u_time", UniformValue::Float(params.time));
    plan.set("u_alpha", UniformValue::Float(params.alpha));

    let region = params
        .region
        .unwrap_or_else(ShaderTextureRegion::blockbuild_fallback);
    plan.set("u_uv", UniformValue::Vec2([region.u, region.v]));
    plan.set("u_uv2", UniformValue::Vec2([region.u2, region.v2]));
    plan.set(
        "u_texsize",
        UniformValue::Vec2([region.texture_size.width, region.texture_size.height]),
    );
}

fn apply_unit_build(plan: &mut ShaderApplyPlan, params: &ShaderParameters) {
    plan.set("u_time", UniformValue::Float(params.time));
    plan.set("u_color", UniformValue::Vec4(params.color.as_vec4()));
    plan.set("u_progress", UniformValue::Float(params.progress));
    apply_required_region(plan, params);
}

fn apply_unit_armor(plan: &mut ShaderApplyPlan, params: &ShaderParameters) {
    plan.set("u_time", UniformValue::Float(params.time));
    plan.set("u_progress", UniformValue::Float(params.progress));
    apply_required_region(plan, params);
}

fn apply_required_region(plan: &mut ShaderApplyPlan, params: &ShaderParameters) {
    if let Some(region) = params.region {
        plan.set("u_uv", UniformValue::Vec2([region.u, region.v]));
        plan.set("u_uv2", UniformValue::Vec2([region.u2, region.v2]));
        plan.set(
            "u_texsize",
            UniformValue::Vec2([region.texture_size.width, region.texture_size.height]),
        );
    } else {
        plan.error(
            ShaderPlanErrorKind::MissingRegion,
            "unit build/armor shaders require TextureRegion metadata",
        );
    }
}

fn apply_screenspace_effect(plan: &mut ShaderApplyPlan, context: &ShaderApplyContext) {
    if context.dp <= 0.0 {
        plan.error(
            ShaderPlanErrorKind::InvalidScale,
            "Scl.scl(1f) equivalent must be positive",
        );
        return;
    }

    let Some(camera) = context.camera else {
        plan.error(
            ShaderPlanErrorKind::MissingCamera,
            "screenspace effect shader requires camera state",
        );
        return;
    };
    if !camera.viewport.valid() {
        plan.error(
            ShaderPlanErrorKind::InvalidViewport,
            "camera viewport dimensions must be positive",
        );
        return;
    }

    let lower_left = camera.lower_left();
    let inv = camera.viewport.inverse().expect("validated viewport");
    plan.set("u_dp", UniformValue::Float(context.dp));
    plan.set("u_time", UniformValue::Float(context.time / context.dp));
    plan.set("u_offset", UniformValue::Vec2(lower_left));
    plan.set(
        "u_texsize",
        UniformValue::Vec2([camera.viewport.width, camera.viewport.height]),
    );
    plan.set("u_invsize", UniformValue::Vec2(inv));
}

fn apply_surface(plan: &mut ShaderApplyPlan, context: &ShaderApplyContext, space: bool) {
    let Some(camera) = context.camera else {
        plan.error(
            ShaderPlanErrorKind::MissingCamera,
            "surface shader requires camera state",
        );
        return;
    };
    if !camera.viewport.valid() {
        plan.error(
            ShaderPlanErrorKind::InvalidViewport,
            "camera viewport dimensions must be positive",
        );
        return;
    }

    if space {
        let Some(graphics) = context.graphics else {
            plan.error(
                ShaderPlanErrorKind::MissingGraphics,
                "space shader requires graphics resolution",
            );
            return;
        };
        if !graphics.valid() {
            plan.error(
                ShaderPlanErrorKind::InvalidViewport,
                "graphics resolution dimensions must be positive",
            );
            return;
        }
        plan.set(
            "u_campos",
            UniformValue::Vec2([camera.position[0], camera.position[1]]),
        );
        plan.set("u_ccampos", UniformValue::Vec3(camera.position));
        plan.set(
            "u_resolution",
            UniformValue::Vec2([graphics.width, graphics.height]),
        );
        plan.set("u_time", UniformValue::Float(context.time));

        if let Some(stars) = &context.stars_texture {
            plan.bind("u_stars", 1, TextureBinding::Asset(stars.clone()));
            plan.set("u_stars", UniformValue::Int(1));
        } else {
            plan.error(
                ShaderPlanErrorKind::MissingTexture,
                "space shader requires sprites/space.png texture",
            );
        }

        if context.effect_buffer_texture.is_some() {
            plan.bind("effectBuffer", 0, TextureBinding::EffectBuffer);
        } else {
            plan.error(
                ShaderPlanErrorKind::MissingTexture,
                "space shader requires renderer effect buffer texture",
            );
        }
    } else {
        plan.set("u_campos", UniformValue::Vec2(camera.lower_left()));
        plan.set(
            "u_resolution",
            UniformValue::Vec2([camera.viewport.width, camera.viewport.height]),
        );
        plan.set("u_time", UniformValue::Float(context.time));

        if let Some(noise) = &context.noise_texture {
            plan.bind("u_noise", 1, TextureBinding::Asset(noise.clone()));
            plan.bind("effectBuffer", 0, TextureBinding::EffectBuffer);
            plan.set_if_present("u_noise", UniformValue::Int(1));
        } else {
            plan.error(
                ShaderPlanErrorKind::MissingTexture,
                "surface shader requires noise texture when u_noise exists",
            );
        }
    }
}

fn apply_planet(plan: &mut ShaderApplyPlan, context: &ShaderApplyContext) {
    let params = &context.parameters;
    plan.set("u_lightdir", UniformValue::Vec3(params.light_dir));
    plan.set(
        "u_ambientColor",
        UniformValue::Vec3(params.ambient_color.as_vec3()),
    );
    plan.set("u_camdir", UniformValue::Vec3(params.cam_dir));
    plan.set("u_campos", UniformValue::Vec3(params.cam_pos));
    plan.set(
        "u_emissive",
        UniformValue::Float(if params.emissive { 1.0 } else { 0.0 }),
    );
}

fn apply_clouds(plan: &mut ShaderApplyPlan, context: &ShaderApplyContext) {
    let params = &context.parameters;
    plan.set("u_alpha", UniformValue::Float(params.cloud_alpha));
    plan.set("u_emissive", UniformValue::Float(0.0));
    plan.set("u_lightdir", UniformValue::Vec3(params.light_dir));
    plan.set(
        "u_ambientColor",
        UniformValue::Vec3(params.ambient_color.as_vec3()),
    );
}

fn apply_atmosphere(plan: &mut ShaderApplyPlan, context: &ShaderApplyContext) {
    let Some(camera) = context.camera else {
        plan.error(
            ShaderPlanErrorKind::MissingCamera,
            "atmosphere shader requires 3D camera state",
        );
        return;
    };
    let Some(graphics) = context.graphics else {
        plan.error(
            ShaderPlanErrorKind::MissingGraphics,
            "atmosphere shader requires graphics resolution",
        );
        return;
    };
    let Some(planet) = &context.parameters.planet else {
        plan.error(
            ShaderPlanErrorKind::MissingPlanet,
            "atmosphere shader requires planet state",
        );
        return;
    };
    if !graphics.valid() {
        plan.error(
            ShaderPlanErrorKind::InvalidViewport,
            "graphics resolution dimensions must be positive",
        );
        return;
    }

    plan.set(
        "u_resolution",
        UniformValue::Vec2([graphics.width, graphics.height]),
    );
    plan.set("u_time", UniformValue::Float(context.global_time / 10.0));
    plan.set("u_campos", UniformValue::Vec3(camera.position));
    plan.set(
        "u_rcampos",
        UniformValue::Vec3([
            camera.position[0] - planet.position[0],
            camera.position[1] - planet.position[1],
            camera.position[2] - planet.position[2],
        ]),
    );
    plan.set("u_light", UniformValue::Vec3(planet.light_normal));
    plan.set(
        "u_color",
        UniformValue::Vec3(planet.atmosphere_color.as_vec3()),
    );
    plan.set(
        "u_innerRadius",
        UniformValue::Float(planet.radius + planet.atmosphere_rad_in),
    );
    plan.set(
        "u_outerRadius",
        UniformValue::Float(planet.radius + planet.atmosphere_rad_out),
    );
    plan.set("u_model", UniformValue::Mat4(planet.model_matrix.clone()));
    plan.set(
        "u_projection",
        UniformValue::Mat4(planet.projection_matrix.clone()),
    );
    plan.set(
        "u_invproj",
        UniformValue::Mat4(planet.inverse_projection_matrix.clone()),
    );
}

fn apply_shockwave(plan: &mut ShaderApplyPlan, context: &ShaderApplyContext) {
    let count = context.parameters.shockwaves.entries.len();
    plan.set("u_shockwave_count", UniformValue::Int(count as i32));

    if count == 0 {
        return;
    }

    let Some(camera) = context.camera else {
        plan.error(
            ShaderPlanErrorKind::MissingCamera,
            "shockwave shader requires camera state",
        );
        return;
    };
    if !camera.viewport.valid() {
        plan.error(
            ShaderPlanErrorKind::InvalidViewport,
            "camera viewport dimensions must be positive",
        );
        return;
    }

    plan.set(
        "u_resolution",
        UniformValue::Vec2([camera.viewport.width, camera.viewport.height]),
    );
    plan.set("u_campos", UniformValue::Vec2(camera.lower_left()));
    plan.set(
        "u_shockwaves",
        UniformValue::Vec4Array(context.parameters.shockwaves.uniforms()),
    );
    plan.operations
        .push(ShaderApplyOperation::ShockwavePreDraw {
            buffer: "shockwave.framebuffer",
            clear_color: ShaderColor::CLEAR,
        });
    plan.operations
        .push(ShaderApplyOperation::ShockwavePostDraw {
            buffer: "shockwave.framebuffer",
            blend_disabled: true,
        });
}

fn default_uniforms_for(shader: ShaderId, params: &ShaderParameters) -> Vec<UniformBinding> {
    let mut context = ShaderApplyContext::default();
    context.camera = Some(ShaderCamera::new(0.0, 0.0, 1.0, 1.0));
    context.graphics = Some(ShaderViewport::new(1.0, 1.0));
    context.parameters = params.clone();
    ShaderCatalog::apply_plan(shader, &context)
        .uniforms()
        .cloned()
        .collect()
}

fn texture_requests_for(shader: ShaderId) -> Vec<ShaderTextureRequest> {
    let mut textures = Vec::new();
    if let Some(texture_name) = shader.texture_name() {
        textures.push(ShaderTextureRequest::noise(texture_name));
    }
    if shader == ShaderId::Space {
        textures.push(ShaderTextureRequest::stars());
    }
    textures
}

fn canonical_name(name: &str) -> String {
    name.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

const fn normalized_one() -> [f32; 3] {
    [0.57735026, 0.57735026, 0.57735026]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform<'a>(plan: &'a ShaderApplyPlan, name: &str) -> &'a UniformBinding {
        plan.uniforms()
            .find(|binding| binding.name == name)
            .unwrap_or_else(|| panic!("missing uniform {name}"))
    }

    #[test]
    fn init_plan_preserves_upstream_names_and_sources() {
        let plan = ShaderCatalog::init_plan();
        assert!(plan.errors.is_empty());

        let enabled: Vec<_> = plan
            .enabled_tasks()
            .map(|task| task.shader.field_name())
            .collect();
        assert_eq!(
            enabled,
            vec![
                "mesh",
                "blockbuild",
                "shield",
                "fog",
                "buildBeam",
                "build",
                "armor",
                "darkness",
                "light",
                "water",
                "arkycite",
                "mud",
                "tar",
                "slag",
                "cryofluid",
                "space",
                "caustics",
                "planet",
                "clouds",
                "planetGrid",
                "atmosphere",
                "unlit",
                "unlitWhite",
                "screenspace",
            ]
        );

        let unlit = ShaderCatalog::descriptor(ShaderId::Unlit);
        assert_eq!(unlit.source.vertex, "unlit.vert");
        assert_eq!(unlit.source.fragment, "planet.frag");

        let shield = ShaderCatalog::descriptor(ShaderId::Shield);
        assert!(shield.optional_load);

        let shockwave = plan
            .tasks
            .iter()
            .find(|task| task.shader == ShaderId::Shockwave)
            .expect("shockwave modeled");
        assert!(!shockwave.enabled);
    }

    #[test]
    fn surface_texture_requests_cover_noise_caustics_and_space_assets() {
        let water = ShaderCatalog::descriptor(ShaderId::Water).texture_requests();
        assert_eq!(water.len(), 1);
        assert_eq!(water[0].path, "sprites/noise.png");
        assert_eq!(water[0].wrap_u, TextureWrap::Repeat);

        let caustics = ShaderCatalog::descriptor(ShaderId::Caustics).texture_requests();
        assert_eq!(caustics[0].path, "sprites/caustics.png");

        let space = ShaderCatalog::descriptor(ShaderId::Space).texture_requests();
        assert_eq!(space.len(), 2);
        assert_eq!(space[0].path, "sprites/noise.png");
        assert_eq!(space[1].path, "sprites/space.png");
        assert_eq!(space[1].wrap_u, TextureWrap::MirroredRepeat);
        assert_eq!(space[1].min_filter, TextureFilter::MipMapLinearLinear);
        assert!(space[1].gen_mip_maps);
    }

    #[test]
    fn blockbuild_defaults_match_upstream_fallback_region() {
        let plan = ShaderCatalog::apply_plan(
            ShaderId::BlockBuild,
            &ShaderApplyContext {
                parameters: ShaderParameters {
                    progress: 0.25,
                    time: 7.0,
                    ..ShaderParameters::default()
                },
                ..ShaderApplyContext::default()
            },
        );
        assert!(!plan.has_errors());
        assert_eq!(
            uniform(&plan, "u_progress").value,
            UniformValue::Float(0.25)
        );
        assert_eq!(uniform(&plan, "u_time").value, UniformValue::Float(7.0));
        assert_eq!(uniform(&plan, "u_alpha").value, UniformValue::Float(1.0));
        assert_eq!(uniform(&plan, "u_uv").value, UniformValue::Vec2([0.0, 0.0]));
        assert_eq!(
            uniform(&plan, "u_uv2").value,
            UniformValue::Vec2([1.0, 1.0])
        );
        assert_eq!(
            uniform(&plan, "u_texsize").value,
            UniformValue::Vec2([1.0, 1.0])
        );
    }

    #[test]
    fn unit_build_collects_missing_region_error_and_keeps_other_uniforms() {
        let plan = ShaderCatalog::apply_plan(
            ShaderId::Build,
            &ShaderApplyContext {
                parameters: ShaderParameters {
                    progress: 0.5,
                    time: 3.0,
                    color: ShaderColor::new(0.1, 0.2, 0.3, 0.4),
                    ..ShaderParameters::default()
                },
                ..ShaderApplyContext::default()
            },
        );

        assert_eq!(plan.errors.len(), 1);
        assert_eq!(plan.errors[0].kind, ShaderPlanErrorKind::MissingRegion);
        assert_eq!(uniform(&plan, "u_time").value, UniformValue::Float(3.0));
        assert_eq!(
            uniform(&plan, "u_color").value,
            UniformValue::Vec4([0.1, 0.2, 0.3, 0.4])
        );
    }

    #[test]
    fn screenspace_effect_builds_uniform_plan_from_camera_and_scale() {
        let plan = ShaderCatalog::apply_plan(
            ShaderId::Shield,
            &ShaderApplyContext {
                camera: Some(ShaderCamera::new(100.0, 80.0, 50.0, 40.0)),
                time: 8.0,
                dp: 2.0,
                ..ShaderApplyContext::default()
            },
        );

        assert!(!plan.has_errors());
        assert_eq!(uniform(&plan, "u_dp").value, UniformValue::Float(2.0));
        assert_eq!(uniform(&plan, "u_time").value, UniformValue::Float(4.0));
        assert_eq!(
            uniform(&plan, "u_offset").value,
            UniformValue::Vec2([75.0, 60.0])
        );
        assert_eq!(
            uniform(&plan, "u_invsize").value,
            UniformValue::Vec2([0.02, 0.025])
        );
    }

    #[test]
    fn surface_and_space_apply_plans_are_backend_neutral() {
        let surface = ShaderCatalog::apply_plan(
            ShaderId::Water,
            &ShaderApplyContext {
                camera: Some(ShaderCamera::new(10.0, 20.0, 100.0, 80.0)),
                time: 11.0,
                ..ShaderApplyContext::default()
            },
        );
        assert!(!surface.has_errors());
        assert_eq!(
            uniform(&surface, "u_campos").value,
            UniformValue::Vec2([-40.0, -20.0])
        );
        assert!(surface.operations.iter().any(|operation| matches!(
            operation,
            ShaderApplyOperation::BindTexture {
                uniform: "u_noise",
                slot: 1,
                texture: TextureBinding::Asset(path),
            } if path == "sprites/noise.png"
        )));
        assert!(surface.operations.iter().any(|operation| matches!(
            operation,
            ShaderApplyOperation::SetUniformIfPresent(binding)
                if binding.name == "u_noise" && binding.value == UniformValue::Int(1)
        )));

        let space = ShaderCatalog::apply_plan(
            ShaderId::Space,
            &ShaderApplyContext {
                camera: Some(ShaderCamera::new(10.0, 20.0, 100.0, 80.0)),
                graphics: Some(ShaderViewport::new(1920.0, 1080.0)),
                time: 12.0,
                ..ShaderApplyContext::default()
            },
        );
        assert!(!space.has_errors());
        assert_eq!(
            uniform(&space, "u_campos").value,
            UniformValue::Vec2([10.0, 20.0])
        );
        assert_eq!(
            uniform(&space, "u_resolution").value,
            UniformValue::Vec2([1920.0, 1080.0])
        );
        assert!(space.operations.iter().any(|operation| matches!(
            operation,
            ShaderApplyOperation::BindTexture {
                uniform: "u_stars",
                slot: 1,
                texture: TextureBinding::Asset(path),
            } if path == "sprites/space.png"
        )));
    }

    #[test]
    fn atmosphere_collects_required_state_and_uniforms() {
        let mut planet = ShaderPlanetState::default();
        planet.position = [1.0, 2.0, 3.0];
        planet.radius = 10.0;
        planet.atmosphere_rad_in = 2.0;
        planet.atmosphere_rad_out = 5.0;
        planet.atmosphere_color = ShaderColor::rgb(0.2, 0.3, 0.4);

        let plan = ShaderCatalog::apply_plan(
            ShaderId::Atmosphere,
            &ShaderApplyContext {
                camera: Some(ShaderCamera {
                    position: [6.0, 8.0, 10.0],
                    viewport: ShaderViewport::new(100.0, 100.0),
                    direction: [0.0, 0.0, -1.0],
                }),
                graphics: Some(ShaderViewport::new(800.0, 600.0)),
                global_time: 50.0,
                parameters: ShaderParameters {
                    planet: Some(planet),
                    ..ShaderParameters::default()
                },
                ..ShaderApplyContext::default()
            },
        );

        assert!(!plan.has_errors());
        assert_eq!(uniform(&plan, "u_time").value, UniformValue::Float(5.0));
        assert_eq!(
            uniform(&plan, "u_rcampos").value,
            UniformValue::Vec3([5.0, 6.0, 7.0])
        );
        assert_eq!(
            uniform(&plan, "u_innerRadius").value,
            UniformValue::Float(12.0)
        );
        assert_eq!(
            uniform(&plan, "u_outerRadius").value,
            UniformValue::Float(15.0)
        );
    }

    #[test]
    fn shockwave_state_caps_updates_and_emits_apply_uniforms() {
        let mut shockwaves = ShockwaveState::new();
        for index in 0..(ShockwaveState::MAX + 1) {
            shockwaves.add_with_lifetime(index as f32, 2.0, 3.0, 10.0);
        }
        assert_eq!(shockwaves.entries.len(), ShockwaveState::MAX);
        assert_eq!(shockwaves.entries[0].x, ShockwaveState::MAX as f32);

        shockwaves.update(5.0, false, false);
        assert_eq!(shockwaves.entries[0].life, 0.5);

        let plan = ShaderCatalog::apply_plan(
            ShaderId::Shockwave,
            &ShaderApplyContext {
                camera: Some(ShaderCamera::new(10.0, 20.0, 100.0, 80.0)),
                parameters: ShaderParameters {
                    shockwaves,
                    ..ShaderParameters::default()
                },
                ..ShaderApplyContext::default()
            },
        );

        assert!(!plan.has_errors());
        assert_eq!(
            uniform(&plan, "u_shockwave_count").value,
            UniformValue::Int(ShockwaveState::MAX as i32)
        );
        assert!(matches!(
            uniform(&plan, "u_shockwaves").value,
            UniformValue::Vec4Array(_)
        ));
        assert!(plan
            .operations
            .iter()
            .any(|operation| matches!(operation, ShaderApplyOperation::ShockwavePreDraw { .. })));
        assert!(plan
            .operations
            .iter()
            .any(|operation| matches!(operation, ShaderApplyOperation::ShockwavePostDraw { .. })));
    }

    #[test]
    fn reload_plan_recreates_enabled_shader_programs_only() {
        let reload = ShaderCatalog::reload_plan();
        assert!(reload.errors.is_empty());
        assert!(reload
            .actions
            .contains(&ShaderReloadAction::DropCachedProgram(ShaderId::BlockBuild)));
        assert!(!reload
            .actions
            .contains(&ShaderReloadAction::DropCachedProgram(ShaderId::Shockwave)));

        let recreate_count = reload
            .actions
            .iter()
            .filter(|action| matches!(action, ShaderReloadAction::Recreate(_)))
            .count();
        assert_eq!(recreate_count, ShaderId::INIT_ORDER.len());
    }

    #[test]
    fn shader_name_lookup_accepts_java_aliases() {
        assert_eq!(ShaderId::from_name("buildBeam"), Some(ShaderId::BuildBeam));
        assert_eq!(
            ShaderId::from_name("planet_grid"),
            Some(ShaderId::PlanetGrid)
        );
        assert_eq!(ShaderId::from_name("unitbuild"), Some(ShaderId::Build));
        assert_eq!(
            ShaderId::from_name("unlit-white"),
            Some(ShaderId::UnlitWhite)
        );
        assert_eq!(ShaderId::from_name("missing"), None);
    }
}
