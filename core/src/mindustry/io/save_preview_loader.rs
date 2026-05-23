//! Side-effect-free mirror of upstream `mindustry.io.SavePreviewLoader`.
//!
//! Java delegates preview loading to `TextureLoader`, but always resolves the
//! sibling file without the trailing `.spreview` extension and deletes that
//! sibling when async loading fails.  This Rust layer keeps the path and
//! recovery decisions explicit so runtime asset adapters can provide actual
//! texture loading and file deletion.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavePreviewLoadTarget {
    pub requested_file: PathBuf,
    pub source_file: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavePreviewFailurePlan {
    DeleteSourceFile(PathBuf),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SavePreviewLoader;

impl SavePreviewLoader {
    pub fn source_file(file: impl AsRef<Path>) -> PathBuf {
        resolve_sibling_without_last_extension(file.as_ref())
    }

    pub fn load_target(file: impl AsRef<Path>) -> SavePreviewLoadTarget {
        let requested_file = file.as_ref().to_path_buf();
        SavePreviewLoadTarget {
            source_file: Self::source_file(&requested_file),
            requested_file,
        }
    }

    pub fn failure_plan(file: impl AsRef<Path>) -> SavePreviewFailurePlan {
        SavePreviewFailurePlan::DeleteSourceFile(Self::source_file(file))
    }
}

pub fn resolve_sibling_without_last_extension(file: &Path) -> PathBuf {
    let stem = file
        .file_stem()
        .map(|stem| stem.to_os_string())
        .unwrap_or_default();
    file.with_file_name(stem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::io::save::SaveSlotRecord;

    #[test]
    fn loader_resolves_preview_source_like_java_texture_loader_delegate() {
        let target = SavePreviewLoader::load_target("previews/save_slot_7.png.spreview");

        assert_eq!(
            target,
            SavePreviewLoadTarget {
                requested_file: PathBuf::from("previews/save_slot_7.png.spreview"),
                source_file: PathBuf::from("previews/save_slot_7.png"),
            }
        );
    }

    #[test]
    fn failure_plan_deletes_resolved_source_file_like_java() {
        assert_eq!(
            SavePreviewLoader::failure_plan("previews/save_slot_7.png.spreview"),
            SavePreviewFailurePlan::DeleteSourceFile(PathBuf::from("previews/save_slot_7.png"))
        );
    }

    #[test]
    fn loader_matches_save_slot_preview_layout_roundtrip() {
        let slot = SaveSlotRecord::new("7.msav");
        let requested = slot.load_preview_file("previews");

        assert_eq!(
            requested,
            PathBuf::from("previews/save_slot_7.png.spreview")
        );
        assert_eq!(
            SavePreviewLoader::source_file(&requested),
            PathBuf::from("previews/save_slot_7.png")
        );
    }

    #[test]
    fn files_without_extra_extension_stay_on_same_sibling_name() {
        assert_eq!(
            SavePreviewLoader::source_file("previews/save_slot_7.png"),
            PathBuf::from("previews/save_slot_7")
        );
        assert_eq!(
            resolve_sibling_without_last_extension(Path::new("preview")),
            PathBuf::from("preview")
        );
    }
}
