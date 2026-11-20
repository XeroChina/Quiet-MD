//! 便携版的启动器: 放在 exe 目录的根,真正的程序在 `Program/`。
//! `-r` 只注册 `.md` / `.markdown` 的文件关联,不启动程序。
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
fn main() {
    use fdl_core::vfs::PathExt as _;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(root) = std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from)) else { return };
    if args.iter().any(|a| a == "-r" || a == "--register") { register(&root); return }
    let app = root.join("Program").join("quiet_md.exe");
    if !app.fs_is_file() { return }
    let mut cmd = Command::new(app);
    cmd.args(&args);
    // 标记在根目录才进便携模式,状态写进 `<根>/Data`。
    if root.join("portable").fs_is_file() { cmd.env("QUIET_MD_PORTABLE_DIR", root.join("Data")); }
    let _ = cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
}

#[cfg(not(target_os = "windows"))]
fn main() {}

/// 把根目录的 `quiet_md.exe` 登记为 `.md` / `.markdown` 的打开方式。Windows 10
/// 起不允许程序静默取代用户已选的默认程序,所以这里只做关联登记: 已选过默认的
/// 扩展仍归原程序,Quiet.md 会出现在「打开方式」和「设置 › 默认应用」里。
#[cfg(target_os = "windows")]
fn register(root: &std::path::Path) {
    use windows_registry::CURRENT_USER;

    let quoted = format!("\"{}\"", root.join("quiet_md.exe").display());
    let icon = format!("{quoted},0");
    let command = format!("{quoted} \"%1\"");
    let progid = "QuietMd.Markdown";

    if let Ok(key) = CURRENT_USER.create(r"Software\Classes\QuietMd.Markdown") {
        let _ = key.set_string("", "Quiet.md");
        if let Ok(k) = key.create("DefaultIcon") { let _ = k.set_string("", &icon); }
        if let Ok(k) = key.create(r"shell\open\command") { let _ = k.set_string("", command); }
    }
    for ext in [".md", ".markdown"] {
        if let Ok(k) = CURRENT_USER.create(format!(r"Software\Classes\{ext}\OpenWithProgids")) { let _ = k.set_string(progid, ""); }
        if let Ok(k) = CURRENT_USER.create(format!(r"Software\Classes\{ext}")) { let _ = k.set_string("", progid); }
    }
    if let Ok(key) = CURRENT_USER.create(r"Software\QuietMd\App\Capabilities") {
        let _ = key.set_string("ApplicationName", "Quiet.md");
        let _ = key.set_string("ApplicationDescription", "A fast, quiet, local notes app");
        let _ = key.set_string("ApplicationIcon", icon);
        if let Ok(k) = key.create("FileAssociations") {
            let _ = k.set_string(".md", progid);
            let _ = k.set_string(".markdown", progid);
        }
    }
    if let Ok(key) = CURRENT_USER.create(r"Software\RegisteredApplications") {
        let _ = key.set_string("QuietMd", r"Software\QuietMd\App\Capabilities");
    }
    notify();
}

/// 让资源管理器立刻采用新的关联,不用重启。
#[cfg(target_os = "windows")]
fn notify() {
    #[link(name = "shell32")]
    unsafe extern "system" { fn SHChangeNotify(evt: i32, flags: u32, a: *const core::ffi::c_void, b: *const core::ffi::c_void); }
    // SHCNE_ASSOCCHANGED
    unsafe { SHChangeNotify(0x0800_0000, 0, core::ptr::null(), core::ptr::null()) }
}
