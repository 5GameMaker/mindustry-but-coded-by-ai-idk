//! Side-effect-free mirror of upstream `mindustry.net.CrashHandler`.
//!
//! Java `CrashHandler` mixes crash report formatting, mod-cause inference,
//! manual-save attempts, local file writes, network disposal and process exit.
//! This Rust layer keeps the report/cause logic pure and exposes the runtime
//! cleanup sequence as an explicit action plan.

use crate::mindustry::core::version::VersionInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashLoadedMod {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub main_class: Option<String>,
    pub enabled: bool,
    pub supported: bool,
}

impl CrashLoadedMod {
    pub fn new(
        name: impl Into<String>,
        display_name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            display_name: display_name.into(),
            version: version.into(),
            main_class: None,
            enabled: true,
            supported: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashStackFrame {
    pub class_name: String,
    pub file_name: Option<String>,
}

impl CrashStackFrame {
    pub fn new(class_name: impl Into<String>, file_name: Option<impl Into<String>>) -> Self {
        Self {
            class_name: class_name.into(),
            file_name: file_name.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashSystemInfo {
    pub os_name: String,
    pub os_arch: String,
    pub os_arch_bits: Option<String>,
    pub gl_version: Option<String>,
    pub android_api_level: Option<i32>,
    pub java_version: String,
    pub max_memory_mb: u64,
    pub cores: usize,
}

impl CrashSystemInfo {
    pub fn new(
        os_name: impl Into<String>,
        os_arch: impl Into<String>,
        os_arch_bits: Option<impl Into<String>>,
        java_version: impl Into<String>,
        max_memory_mb: u64,
        cores: usize,
    ) -> Self {
        Self {
            os_name: os_name.into(),
            os_arch: os_arch.into(),
            os_arch_bits: os_arch_bits.map(Into::into),
            gl_version: None,
            android_api_level: None,
            java_version: java_version.into(),
            max_memory_mb,
            cores,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReportContext {
    pub version: VersionInfo,
    pub date_time: String,
    pub system: CrashSystemInfo,
    pub headless: bool,
    pub report_issue_url: Option<String>,
    pub mods_initialized: bool,
    pub mods: Vec<CrashLoadedMod>,
    pub cause_mod: Option<CrashLoadedMod>,
    pub patches: Vec<String>,
}

impl CrashReportContext {
    pub fn enabled_supported_mods(&self) -> Vec<&CrashLoadedMod> {
        self.mods
            .iter()
            .filter(|module| module.enabled && module.supported)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashHandleRuntime {
    pub username: String,
    pub app_data_dir: String,
    pub crash_timestamp: String,
}

impl CrashHandleRuntime {
    pub fn new(
        username: impl Into<String>,
        app_data_dir: impl Into<String>,
        crash_timestamp: impl Into<String>,
    ) -> Self {
        Self {
            username: username.into(),
            app_data_dir: normalize_path(app_data_dir.into()),
            crash_timestamp: crash_timestamp.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashHandlerAction {
    LogException,
    ManualSave,
    Exit { code: i32 },
    LoadVersionProperties,
    WriteCrashReport { path: String, contents: String },
    NotifyWriteListener { path: String },
    DisposeNet,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CrashHandlerPlan {
    pub actions: Vec<CrashHandlerAction>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CrashHandler;

impl CrashHandler {
    pub fn create_report(context: &CrashReportContext, exception_text: &str) -> String {
        create_report(context, exception_text)
    }

    pub fn handle_plan(
        context: &CrashReportContext,
        runtime: &CrashHandleRuntime,
        exception_text: &str,
    ) -> CrashHandlerPlan {
        handle_plan(context, runtime, exception_text)
    }

    pub fn get_mod_cause(
        stack_trace: &[CrashStackFrame],
        mods: &[CrashLoadedMod],
    ) -> Option<CrashLoadedMod> {
        detect_mod_cause(stack_trace, mods)
    }
}

pub fn create_report(context: &CrashReportContext, exception_text: &str) -> String {
    let mut report = if let Some(cause) = &context.cause_mod {
        format!(
            "The mod '{}' ({}) has caused Mindustry to crash.\n",
            cause.display_name, cause.name
        )
    } else {
        "Mindustry has crashed. How unfortunate.\n".to_string()
    };

    if context.mods_initialized && context.mods.is_empty() && context.version.build != -1 {
        if let Some(issue_url) = &context.report_issue_url {
            report.push_str(&format!("Report this at {issue_url}\n\n"));
        }
    }

    report.push_str("Version: ");
    report.push_str(&context.version.combined());
    if context.version.build_date != "unknown" {
        report.push_str(" (Built ");
        report.push_str(&context.version.build_date);
        report.push(')');
    }
    if context.headless {
        report.push_str(" (Server)");
    }
    report.push('\n');

    report.push_str("Date: ");
    report.push_str(&context.date_time);
    report.push('\n');

    report.push_str("OS: ");
    report.push_str(&context.system.os_name);
    if let Some(bits) = &context.system.os_arch_bits {
        report.push_str(" x");
        report.push_str(bits);
    }
    report.push_str(" (");
    report.push_str(&context.system.os_arch);
    report.push_str(")\n");

    if let Some(gl_version) = &context.system.gl_version {
        report.push_str("GL Version: ");
        report.push_str(gl_version);
        report.push('\n');
    }

    if let Some(api_level) = context.system.android_api_level {
        report.push_str("Android API level: ");
        report.push_str(&api_level.to_string());
        report.push('\n');
    }

    report.push_str("Java Version: ");
    report.push_str(&context.system.java_version);
    report.push('\n');

    report.push_str("Runtime Available Memory: ");
    report.push_str(&context.system.max_memory_mb.to_string());
    report.push_str("mb\n");

    report.push_str("Cores: ");
    report.push_str(&context.system.cores.to_string());
    report.push('\n');

    if let Some(cause) = &context.cause_mod {
        report.push_str("Likely Cause: ");
        report.push_str(&cause.display_name);
        report.push_str(" (");
        report.push_str(&cause.name);
        report.push_str(" v");
        report.push_str(&cause.version);
        report.push_str(")\n");
    }

    report.push_str("Mods: ");
    if !context.mods_initialized {
        report.push_str("<no mod init>");
    } else {
        let enabled = context
            .enabled_supported_mods()
            .into_iter()
            .map(|module| format!("{}:{}", module.name, module.version))
            .collect::<Vec<_>>();
        if enabled.is_empty() {
            report.push_str("none (vanilla)");
        } else {
            report.push_str(&enabled.join(", "));
        }
    }
    report.push('\n');

    if !context.patches.is_empty() {
        report.push_str("Patches: \n");
        report.push_str(&context.patches.join("\n---\n"));
        report.push('\n');
    }

    report.push_str("\n\n");
    report.push_str(exception_text);
    report
}

pub fn handle_plan(
    context: &CrashReportContext,
    runtime: &CrashHandleRuntime,
    exception_text: &str,
) -> CrashHandlerPlan {
    let mut actions = vec![
        CrashHandlerAction::LogException,
        CrashHandlerAction::ManualSave,
    ];

    if runtime.username == "anuke" && context.version.modifier != "steam" {
        actions.push(CrashHandlerAction::Exit { code: 1 });
        return CrashHandlerPlan { actions };
    }

    if context.version.number == 0 {
        actions.push(CrashHandlerAction::LoadVersionProperties);
    }

    let path = crash_report_path(&runtime.app_data_dir, &runtime.crash_timestamp);
    let contents = create_report(context, exception_text);

    actions.push(CrashHandlerAction::WriteCrashReport {
        path: path.clone(),
        contents,
    });
    actions.push(CrashHandlerAction::NotifyWriteListener { path });
    actions.push(CrashHandlerAction::DisposeNet);
    actions.push(CrashHandlerAction::Exit { code: 1 });

    CrashHandlerPlan { actions }
}

pub fn detect_mod_cause(
    stack_trace: &[CrashStackFrame],
    mods: &[CrashLoadedMod],
) -> Option<CrashLoadedMod> {
    for frame in stack_trace {
        if is_system_stack_class(&frame.class_name) {
            continue;
        }

        for module in mods {
            if module.main_class.as_deref().is_some_and(|main_class| {
                match_dotted_prefix_score(main_class, &frame.class_name) > 0
            }) {
                return Some(module.clone());
            }

            if frame.file_name.as_deref().is_some_and(|file_name| {
                file_name.ends_with(".js") && file_name.starts_with(&format!("{}/", module.name))
            }) {
                return Some(module.clone());
            }
        }
    }

    None
}

pub fn match_dotted_prefix_score(name1: &str, name2: &str) -> usize {
    let arr1: Vec<_> = name1.split('.').collect();
    let arr2: Vec<_> = name2.split('.').collect();
    let mut matches = 0usize;

    for index in 0..arr1.len().min(arr2.len()) {
        if arr1[index] != arr2[index] {
            return index;
        } else if !matches!(arr1[index], "net" | "org" | "com" | "io") {
            matches += 1;
        }
    }

    matches
}

pub fn crash_report_path(app_data_dir: &str, timestamp: &str) -> String {
    format!(
        "{}/crashes/crash-report-{}.txt",
        normalize_path(app_data_dir.to_string()),
        timestamp
    )
}

fn is_system_stack_class(class_name: &str) -> bool {
    ["mindustry.", "arc.", "java.", "javax.", "sun.", "jdk."]
        .iter()
        .any(|prefix| class_name.starts_with(prefix))
}

fn normalize_path(path: String) -> String {
    path.trim_end_matches(['/', '\\'])
        .replace('\\', "/")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version_info() -> VersionInfo {
        VersionInfo {
            build_type: "official".into(),
            modifier: "release".into(),
            commit_hash: "abc123".into(),
            build_date: "2025-05-23".into(),
            number: 4,
            build: 157,
            revision: 4,
            is_steam: false,
            enabled: true,
        }
    }

    fn system_info() -> CrashSystemInfo {
        let mut system = CrashSystemInfo::new("Windows", "amd64", Some("64"), "17.0.10", 2048, 16);
        system.gl_version = Some("OpenGL 4.6".into());
        system
    }

    #[test]
    fn create_report_formats_vanilla_report_issue_and_runtime_details() {
        let context = CrashReportContext {
            version: version_info(),
            date_time: "May 23, 2026 12:00:00 PM".into(),
            system: system_info(),
            headless: false,
            report_issue_url: Some("https://example.invalid/issues".into()),
            mods_initialized: true,
            mods: Vec::new(),
            cause_mod: None,
            patches: Vec::new(),
        };

        let report = create_report(&context, "java.lang.RuntimeException: boom");

        assert!(report.starts_with("Mindustry has crashed. How unfortunate.\n"));
        assert!(report.contains("Report this at https://example.invalid/issues"));
        assert!(report.contains("Version: release build 157.4 (abc123) (Built 2025-05-23)"));
        assert!(report.contains("OS: Windows x64 (amd64)"));
        assert!(report.contains("GL Version: OpenGL 4.6"));
        assert!(report.contains("Java Version: 17.0.10"));
        assert!(report.contains("Runtime Available Memory: 2048mb"));
        assert!(report.contains("Mods: none (vanilla)"));
        assert!(report.ends_with("java.lang.RuntimeException: boom"));
    }

    #[test]
    fn create_report_formats_cause_mod_headless_android_and_patches() {
        let mut system = CrashSystemInfo::new("Android", "arm64", Some("64"), "17", 1024, 8);
        system.android_api_level = Some(34);
        let mut module = CrashLoadedMod::new("example-mod", "Example Mod", "1.2.3");
        module.main_class = Some("com.example.mod.Main".into());
        let context = CrashReportContext {
            version: version_info(),
            date_time: "May 23, 2026 12:00:00 PM".into(),
            system,
            headless: true,
            report_issue_url: None,
            mods_initialized: true,
            mods: vec![module.clone()],
            cause_mod: Some(module),
            patches: vec!["patch-a".into(), "patch-b".into()],
        };

        let report = create_report(&context, "stacktrace");

        assert!(report
            .starts_with("The mod 'Example Mod' (example-mod) has caused Mindustry to crash.\n"));
        assert!(
            report.contains("Version: release build 157.4 (abc123) (Built 2025-05-23) (Server)")
        );
        assert!(report.contains("Android API level: 34"));
        assert!(report.contains("Likely Cause: Example Mod (example-mod v1.2.3)"));
        assert!(report.contains("Mods: example-mod:1.2.3"));
        assert!(report.contains("Patches: \npatch-a\n---\npatch-b"));
    }

    #[test]
    fn create_report_marks_missing_mod_init_like_java() {
        let context = CrashReportContext {
            version: version_info(),
            date_time: "May 23, 2026 12:00:00 PM".into(),
            system: system_info(),
            headless: false,
            report_issue_url: Some("https://example.invalid/issues".into()),
            mods_initialized: false,
            mods: Vec::new(),
            cause_mod: None,
            patches: Vec::new(),
        };

        let report = create_report(&context, "stacktrace");

        assert!(report.contains("Mods: <no mod init>"));
        assert!(!report.contains("Report this at"));
    }

    #[test]
    fn detect_mod_cause_prefers_main_class_match() {
        let mut first = CrashLoadedMod::new("first", "First", "1.0");
        first.main_class = Some("com.example.first.Main".into());
        let mut second = CrashLoadedMod::new("second", "Second", "2.0");
        second.main_class = Some("org.other.second.Entry".into());
        let stack = vec![
            CrashStackFrame::new("mindustry.net.Net", None::<String>),
            CrashStackFrame::new("com.example.first.entities.UnitCrash", None::<String>),
        ];

        assert_eq!(
            detect_mod_cause(&stack, &[first.clone(), second]),
            Some(first)
        );
    }

    #[test]
    fn detect_mod_cause_supports_js_file_prefix_like_java() {
        let module = CrashLoadedMod::new("scripting", "Scripting", "3.0");
        let stack = vec![CrashStackFrame::new(
            "custom.ScriptRunner",
            Some("scripting/main.js"),
        )];

        assert_eq!(detect_mod_cause(&stack, &[module.clone()]), Some(module));
    }

    #[test]
    fn match_score_follows_java_prefix_logic() {
        assert_eq!(
            match_dotted_prefix_score("com.example.mod.Main", "com.example.mod.Blocks"),
            3
        );
        assert_eq!(
            match_dotted_prefix_score("org.example.mod.Main", "org.example.other.Blocks"),
            2
        );
        assert_eq!(
            match_dotted_prefix_score("io.test.mod.Main", "net.other.mod.Blocks"),
            0
        );
    }

    #[test]
    fn handle_plan_short_circuits_custom_anuke_builds() {
        let context = CrashReportContext {
            version: VersionInfo {
                modifier: "release".into(),
                ..version_info()
            },
            date_time: "May 23, 2026 12:00:00 PM".into(),
            system: system_info(),
            headless: false,
            report_issue_url: None,
            mods_initialized: true,
            mods: Vec::new(),
            cause_mod: None,
            patches: Vec::new(),
        };
        let runtime = CrashHandleRuntime::new("anuke", "C:/Mindustry", "05_23_2026_12_00_00");

        assert_eq!(
            handle_plan(&context, &runtime, "stacktrace").actions,
            vec![
                CrashHandlerAction::LogException,
                CrashHandlerAction::ManualSave,
                CrashHandlerAction::Exit { code: 1 },
            ]
        );
    }

    #[test]
    fn handle_plan_loads_version_writes_report_and_disposes_net() {
        let context = CrashReportContext {
            version: VersionInfo {
                number: 0,
                ..version_info()
            },
            date_time: "May 23, 2026 12:00:00 PM".into(),
            system: system_info(),
            headless: false,
            report_issue_url: None,
            mods_initialized: true,
            mods: Vec::new(),
            cause_mod: None,
            patches: Vec::new(),
        };
        let runtime = CrashHandleRuntime::new(
            "player",
            "C:/Users/example/AppData/Roaming/Mindustry",
            "05_23_2026_12_00_00",
        );

        let plan = handle_plan(&context, &runtime, "stacktrace");

        assert_eq!(
            plan.actions,
            vec![
                CrashHandlerAction::LogException,
                CrashHandlerAction::ManualSave,
                CrashHandlerAction::LoadVersionProperties,
                CrashHandlerAction::WriteCrashReport {
                    path: "C:/Users/example/AppData/Roaming/Mindustry/crashes/crash-report-05_23_2026_12_00_00.txt".into(),
                    contents: create_report(&context, "stacktrace"),
                },
                CrashHandlerAction::NotifyWriteListener {
                    path: "C:/Users/example/AppData/Roaming/Mindustry/crashes/crash-report-05_23_2026_12_00_00.txt".into(),
                },
                CrashHandlerAction::DisposeNet,
                CrashHandlerAction::Exit { code: 1 },
            ]
        );
    }

    #[test]
    fn crash_report_path_normalizes_separators() {
        assert_eq!(
            crash_report_path("C:\\Mindustry\\", "05_23_2026_12_00_00"),
            "C:/Mindustry/crashes/crash-report-05_23_2026_12_00_00.txt"
        );
    }
}
