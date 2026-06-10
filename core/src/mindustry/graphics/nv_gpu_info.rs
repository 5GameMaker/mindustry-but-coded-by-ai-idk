//! Nvidia memory-info query state mirroring upstream `mindustry.graphics.NvGpuInfo`.

pub const GL_GPU_MEM_INFO_TOTAL_AVAILABLE_MEM_NVX: i32 = 0x9048;
pub const GL_GPU_MEM_INFO_CURRENT_AVAILABLE_MEM_NVX: i32 = 0x9049;
pub const NVX_GPU_MEMORY_INFO_EXTENSION: &str = "GL_NVX_gpu_memory_info";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvGpuInfo {
    pub supported: bool,
    pub initialized: bool,
}

impl NvGpuInfo {
    pub const fn new() -> Self {
        Self {
            supported: false,
            initialized: false,
        }
    }

    pub fn has_memory_info<F>(&mut self, supports_extension: F) -> bool
    where
        F: FnOnce(&str) -> bool,
    {
        if !self.initialized {
            self.supported = supports_extension(NVX_GPU_MEMORY_INFO_EXTENSION);
            self.initialized = true;
        }
        self.supported
    }

    pub fn get_max_memory_kb<F, G>(&mut self, supports_extension: F, get_int: G) -> i32
    where
        F: FnOnce(&str) -> bool,
        G: FnOnce(i32) -> i32,
    {
        if self.has_memory_info(supports_extension) {
            get_int(GL_GPU_MEM_INFO_TOTAL_AVAILABLE_MEM_NVX)
        } else {
            0
        }
    }

    pub fn get_available_memory_kb<F, G>(&mut self, supports_extension: F, get_int: G) -> i32
    where
        F: FnOnce(&str) -> bool,
        G: FnOnce(i32) -> i32,
    {
        if self.has_memory_info(supports_extension) {
            get_int(GL_GPU_MEM_INFO_CURRENT_AVAILABLE_MEM_NVX)
        } else {
            0
        }
    }
}

impl Default for NvGpuInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn nv_gpu_info_checks_extension_once_and_caches_support() {
        let calls = Cell::new(0);
        let mut info = NvGpuInfo::new();

        assert!(info.has_memory_info(|extension| {
            calls.set(calls.get() + 1);
            extension == NVX_GPU_MEMORY_INFO_EXTENSION
        }));
        assert!(info.has_memory_info(|_| {
            calls.set(calls.get() + 1);
            false
        }));

        assert_eq!(calls.get(), 1);
        assert!(info.initialized);
        assert!(info.supported);
    }

    #[test]
    fn nv_gpu_info_returns_zero_when_extension_is_unsupported() {
        let mut info = NvGpuInfo::new();

        assert_eq!(info.get_max_memory_kb(|_| false, |_| 8192), 0);
        assert_eq!(info.get_available_memory_kb(|_| true, |_| 4096), 0);
    }

    #[test]
    fn nv_gpu_info_queries_total_and_available_constants_when_supported() {
        let mut max = NvGpuInfo::new();
        let mut available = NvGpuInfo::new();

        assert_eq!(
            max.get_max_memory_kb(
                |extension| extension == NVX_GPU_MEMORY_INFO_EXTENSION,
                |pname| {
                    assert_eq!(pname, GL_GPU_MEM_INFO_TOTAL_AVAILABLE_MEM_NVX);
                    8192
                },
            ),
            8192
        );
        assert_eq!(
            available.get_available_memory_kb(
                |extension| extension == NVX_GPU_MEMORY_INFO_EXTENSION,
                |pname| {
                    assert_eq!(pname, GL_GPU_MEM_INFO_CURRENT_AVAILABLE_MEM_NVX);
                    4096
                },
            ),
            4096
        );
    }
}
