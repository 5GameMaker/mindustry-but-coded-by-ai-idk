//! Windows Intel GPU launch marker mirroring upstream `mindustry.graphics.IntelGpuCheck`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const INTEL_GPU_MARKER_FILE: &str = "was_intel_gpu";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntelGpuInitAction {
    NotWindows,
    WriteMarker(PathBuf),
    DeleteMarker(PathBuf),
    AlreadyAbsent(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelGpuCheck {
    pub was_intel: bool,
    pub checked_last_launch: bool,
}

impl IntelGpuCheck {
    pub const fn new() -> Self {
        Self {
            was_intel: false,
            checked_last_launch: false,
        }
    }

    pub fn marker_path(app_data_dir: impl AsRef<Path>) -> PathBuf {
        app_data_dir.as_ref().join(INTEL_GPU_MARKER_FILE)
    }

    /// initialize intel version check for the next application launch
    pub fn init(
        app_data_dir: impl AsRef<Path>,
        vendor: &str,
        is_windows: bool,
    ) -> io::Result<IntelGpuInitAction> {
        if !is_windows {
            return Ok(IntelGpuInitAction::NotWindows);
        }

        let path = Self::marker_path(app_data_dir);
        let is_intel = vendor.to_lowercase().contains("intel");
        if is_intel {
            fs::write(&path, "1")?;
            Ok(IntelGpuInitAction::WriteMarker(path))
        } else if path.exists() {
            fs::remove_file(&path)?;
            Ok(IntelGpuInitAction::DeleteMarker(path))
        } else {
            Ok(IntelGpuInitAction::AlreadyAbsent(path))
        }
    }

    /// @return whether the last launch used an intel GPU on Windows
    pub fn was_intel(
        &mut self,
        app_data_dir: impl AsRef<Path>,
        is_windows: bool,
    ) -> io::Result<bool> {
        if !is_windows {
            return Ok(false);
        }
        if self.checked_last_launch {
            return Ok(self.was_intel);
        }
        self.checked_last_launch = true;

        let path = Self::marker_path(app_data_dir);
        self.was_intel = path.exists() && fs::read_to_string(path)? == "1";
        Ok(self.was_intel)
    }
}

impl Default for IntelGpuCheck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rust_mindustry_{name}_{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn intel_gpu_init_writes_marker_only_for_windows_intel_vendor() {
        let dir = temp_dir("intel_write");

        let action = IntelGpuCheck::init(&dir, "Intel(R) UHD Graphics", true).unwrap();

        assert_eq!(
            action,
            IntelGpuInitAction::WriteMarker(IntelGpuCheck::marker_path(&dir))
        );
        assert_eq!(
            fs::read_to_string(IntelGpuCheck::marker_path(&dir)).unwrap(),
            "1"
        );
    }

    #[test]
    fn intel_gpu_init_deletes_marker_for_non_intel_windows_vendor() {
        let dir = temp_dir("intel_delete");
        fs::write(IntelGpuCheck::marker_path(&dir), "1").unwrap();

        let action = IntelGpuCheck::init(&dir, "NVIDIA", true).unwrap();

        assert_eq!(
            action,
            IntelGpuInitAction::DeleteMarker(IntelGpuCheck::marker_path(&dir))
        );
        assert!(!IntelGpuCheck::marker_path(&dir).exists());
    }

    #[test]
    fn intel_gpu_was_intel_reads_once_and_caches_last_launch() {
        let dir = temp_dir("intel_read");
        fs::write(IntelGpuCheck::marker_path(&dir), "1").unwrap();
        let mut check = IntelGpuCheck::new();

        assert!(check.was_intel(&dir, true).unwrap());
        fs::remove_file(IntelGpuCheck::marker_path(&dir)).unwrap();

        assert!(check.was_intel(&dir, true).unwrap());
        assert!(check.checked_last_launch);
    }

    #[test]
    fn intel_gpu_check_is_noop_off_windows() {
        let dir = temp_dir("intel_nowindows");

        assert_eq!(
            IntelGpuCheck::init(&dir, "Intel", false).unwrap(),
            IntelGpuInitAction::NotWindows
        );
        assert!(!IntelGpuCheck::marker_path(&dir).exists());
        assert!(!IntelGpuCheck::new().was_intel(&dir, false).unwrap());
    }
}
