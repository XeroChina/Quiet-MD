//! 便携化: exe 旁放一个空的 `portable` 文件，本机状态就都写在 `<exe>/Data`,
//! 整个目录可以带走。macOS 的 exe 在 `.app` 包内、wasm 是网页演示，`data_dir`
//! 在这两处恒为 `None`。

use std::path::{Path, PathBuf};

#[cfg(all(not(target_os = "macos"), not(target_family = "wasm")))]
use std::sync::OnceLock;

use crate::vfs::PathExt as _;

const MARKER: &str = "portable";

/// `exe` 旁有 `portable` 标记则返回 `<exe 目录>/Data`;纯函数,便于测试。
pub fn data_dir_from(exe: Option<&Path>) -> Option<PathBuf> {
    let dir = exe?.parent()?;
    dir.join(MARKER).fs_is_file().then(|| dir.join("Data"))
}

/// 本进程的 `Data` 目录。macOS 的 exe 在 `.app` 包内、wasm 是网页演示,恒为 `None`。
#[cfg(any(target_os = "macos", target_family = "wasm"))]
pub fn data_dir() -> Option<PathBuf> { None }

/// 本进程的 `Data` 目录,结果缓存。`QUIET_MD_PORTABLE` 存在即强制便携。
#[cfg(all(not(target_os = "macos"), not(target_family = "wasm")))]
pub fn data_dir() -> Option<PathBuf> {
    static DATA: OnceLock<Option<PathBuf>> = OnceLock::new();
    DATA.get_or_init(|| {
        let exe = std::env::current_exe().ok();
        if std::env::var_os("QUIET_MD_PORTABLE").is_some() { return exe.as_deref()?.parent().map(|d| d.join("Data")) }
        data_dir_from(exe.as_deref())
    }).clone()
}

/// 是否在便携模式。
pub fn enabled() -> bool { data_dir().is_some() }

/// 本机状态目录: 便携时 `<exe>/Data/quiet-md`,否则系统的本地应用数据目录。
pub fn state_dir() -> PathBuf {
    data_dir().unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(crate::vfs::temp_dir)).join("quiet-md")
}

/// 缓存目录: 便携时同样落在 `<exe>/Data/quiet-md`,不写系统的缓存目录。
pub fn cache_state_dir() -> PathBuf {
    data_dir().unwrap_or_else(|| dirs::cache_dir().unwrap_or_else(crate::vfs::temp_dir)).join("quiet-md")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_turns_portability_on() {
        let dir = crate::vfs::temp_dir().join(format!("quiet-portable-{}", uuid::Uuid::new_v4()));
        crate::vfs::create_dir_all(&dir).unwrap();
        let exe = dir.join("quiet_md.exe");
        assert_eq!(data_dir_from(Some(&exe)), None);
        crate::vfs::write(&dir.join(MARKER), b"").unwrap();
        assert_eq!(data_dir_from(Some(&exe)), Some(dir.join("Data")));
        let _ = crate::vfs::remove_dir_all(&dir);
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
