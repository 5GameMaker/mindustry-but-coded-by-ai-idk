use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ExportStat {
    pub counter: f32,
    pub means: Vec<f32>,
    pub loaded: bool,
    pub mean: f32,
}

impl ExportStat {
    pub fn new() -> Self {
        Self {
            counter: 0.0,
            means: Vec::new(),
            loaded: false,
            mean: 0.0,
        }
    }
}

impl Default for ExportStat {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectorInfo {
    pub production: HashMap<String, ExportStat>,
    pub raw_production: HashMap<String, ExportStat>,
    pub export: HashMap<String, ExportStat>,
    pub imports: HashMap<String, ExportStat>,
    pub items: Vec<(String, i32)>,
    pub best_core_type: String,
    pub storage_capacity: i32,
    pub has_core: bool,
    pub last_preset_name: Option<String>,
    pub last_width: i32,
    pub last_height: i32,
    pub was_captured: bool,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub resources: Vec<String>,
    pub waves: bool,
    pub attack: bool,
    pub has_spawns: bool,
    pub attempts: i32,
    pub wave: i32,
    pub win_wave: i32,
    pub wave_spacing: f32,
    pub spawn_position: i32,
    pub minutes_captured: f32,
    pub light_coverage: f32,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub content_icon: Option<String>,
    pub wave_version: i32,
    pub shown: bool,
    pub import_cooldown_timers: HashMap<String, f32>,
    pub import_rate_cache: Option<Vec<f32>>,
    pub last_imported: Vec<(String, i32)>,
}

impl Default for SectorInfo {
    fn default() -> Self {
        Self {
            production: HashMap::new(),
            raw_production: HashMap::new(),
            export: HashMap::new(),
            imports: HashMap::new(),
            items: Vec::new(),
            best_core_type: "coreShard".to_string(),
            storage_capacity: 0,
            has_core: true,
            last_preset_name: None,
            last_width: 0,
            last_height: 0,
            was_captured: false,
            origin: None,
            destination: None,
            resources: Vec::new(),
            waves: true,
            attack: false,
            has_spawns: true,
            attempts: 0,
            wave: 1,
            win_wave: -1,
            wave_spacing: 2.0 * 60.0 * 60.0,
            spawn_position: 0,
            minutes_captured: 0.0,
            light_coverage: 0.0,
            name: None,
            icon: None,
            content_icon: None,
            wave_version: -1,
            shown: false,
            import_cooldown_timers: HashMap::new(),
            import_rate_cache: None,
            last_imported: Vec::new(),
        }
    }
}
