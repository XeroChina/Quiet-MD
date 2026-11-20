//! 便携化: exe 目录的根放一个空的 `portable` 文件，本机状态就都写在 `<根>/Data`,
//! 整个目录可以带走。根目录的 `quiet_md.exe` 是启动器,真正的程序在 `Program/`,
//! 所以标记也接受在 exe 的上层目录。macOS 的 exe 在 `.app` 包内、wasm 是网页
//! 演示,`data_dir` 在这两处恒为 `None`。

use std::path::{Path, PathBuf};

#[cfg(all(not(target_os = "macos"), not(target_family = "wasm")))]
use std::sync::OnceLock;

use crate::vfs::PathExt as _;

const MARKER: &str = "portable";

/// 标记所在目录的 `Data`。先看 `exe` 旁,再看它的上层(启动器在根、程序在
/// `Program/`)。纯函数,便于测试。
pub fn data_dir_from(exe: Option<&Path>) -> Option<PathBuf> {
    let dir = exe?.parent()?;
    if dir.join(MARKER).fs_is_file() { return Some(dir.join("Data")) }
    let root = dir.parent()?;
    if root.join(MARKER).fs_is_file() { return Some(root.join("Data")) }
    None
}

/// 本进程的 `Data` 目录。macOS 的 exe 在 `.app` 包内、wasm 是网页演示,恒为 `None`。
#[cfg(any(target_os = "macos", target_family = "wasm"))]
pub fn data_dir() -> Option<PathBuf> { None }

/// 本进程的 `Data` 目录,结果缓存。启动器用 `QUIET_MD_PORTABLE_DIR` 显式指定。
#[cfg(all(not(target_os = "macos"), not(target_family = "wasm")))]
pub fn data_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("QUIET_MD_PORTABLE_DIR") { return Some(PathBuf::from(d)) }
    static DATA: OnceLock<Option<PathBuf>> = OnceLock::new();
    DATA.get_or_init(|| data_dir_from(std::env::current_exe().ok().as_deref())).clone()
}

/// 是否在便携模式。
pub fn enabled() -> bool { data_dir().is_some() }

/// 本机状态目录: 便携时 `<根>/Data/quiet-md`,否则系统的本地应用数据目录。
pub fn state_dir() -> PathBuf {
    data_dir().unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(crate::vfs::temp_dir)).join("quiet-md")
}

/// 缓存目录: 便携时同样落在 `<根>/Data/quiet-md`,不写系统的缓存目录。
pub fn cache_state_dir() -> PathBuf {
    data_dir().unwrap_or_else(|| dirs::cache_dir().unwrap_or_else(crate::vfs::temp_dir)).join("quiet-md")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_beside_the_exe() {
        let dir = crate::vfs::temp_dir().join(format!("quiet-portable-{}", uuid::Uuid::new_v4()));
        crate::vfs::create_dir_all(&dir).unwrap();
        let exe = dir.join("quiet_md.exe");
        assert_eq!(data_dir_from(Some(&exe)), None);
        crate::vfs::write(dir.join(MARKER), b"").unwrap();
        assert_eq!(data_dir_from(Some(&exe)), Some(dir.join("Data")));
        let _ = crate::vfs::remove_dir_all(&dir);
    }

    #[test]
    fn marker_one_level_up() {
        let root = crate::vfs::temp_dir().join(format!("quiet-portable-up-{}", uuid::Uuid::new_v4()));
        let program = root.join("Program");
        crate::vfs::create_dir_all(&program).unwrap();
        let exe = program.join("quiet_md.exe");
        assert_eq!(data_dir_from(Some(&exe)), None);
        crate::vfs::write(root.join(MARKER), b"").unwrap();
        assert_eq!(data_dir_from(Some(&exe)), Some(root.join("Data")));
        let _ = crate::vfs::remove_dir_all(&root);
    }

    #[test]
    fn no_marker_no_data_dir() {
        let dir = crate::vfs::temp_dir().join(format!("quiet-noportable-{}", uuid::Uuid::new_v4()));
        crate::vfs::create_dir_all(&dir).unwrap();
        assert_eq!(data_dir_from(Some(&dir.join("quiet_md.exe"))), None);
        assert_eq!(data_dir_from(None), None);
        let _ = crate::vfs::remove_dir_all(&dir);
    }
}
