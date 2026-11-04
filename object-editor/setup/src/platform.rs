//! Where Tinkerbench goes on each system, and how it joins the system's menus
//! (AGE-TECH-005: platform-specific by requirement, kept in this one module).

#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
pub use unix::*;

#[cfg(not(windows))]
mod unix {
    use std::path::{Path, PathBuf};

    use wc3_install::APP_IDENTIFIER;

    use crate::install::{InstallError, Plan, io_error, write_file};
    use crate::payload;

    pub const BINARY: &str = "tinkerbench";
    const INSTALL_FOLDER: &str = "tinkerbench";
    const EXECUTABLE_MODE: u32 = 0o755;
    const DESKTOP_SHORTCUT: &str = "Tinkerbench.desktop";

    pub fn default_dir() -> Option<PathBuf> {
        dirs::data_local_dir().map(|dir| dir.join(INSTALL_FOLDER))
    }

    pub fn make_executable(path: &Path) -> Result<(), InstallError> {
        use std::os::unix::fs::PermissionsExt;

        std::fs::set_permissions(path, std::fs::Permissions::from_mode(EXECUTABLE_MODE))
            .map_err(io_error("make executable", path))
    }

    /// The menu entry, its icons, the `tinkerbench` command and, if asked, a
    /// desktop shortcut.
    pub fn integrate(
        plan: &Plan,
        binary: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), InstallError> {
        let data = dirs::data_local_dir().ok_or(InstallError::NoUserFolders)?;

        for (size, png) in payload::ICONS {
            let icon = data
                .join("icons/hicolor")
                .join(format!("{size}x{size}"))
                .join("apps")
                .join(format!("{APP_IDENTIFIER}.png"));

            write_file(&icon, png)?;
            files.push(icon);
        }

        let entry = desktop_entry(binary);
        let menu_entry = data
            .join("applications")
            .join(format!("{APP_IDENTIFIER}.desktop"));

        write_file(&menu_entry, entry.as_bytes())?;
        files.push(menu_entry);

        if let Some(bin) = dirs::executable_dir() {
            let command = bin.join(BINARY);

            std::fs::create_dir_all(&bin).map_err(io_error("create", &bin))?;
            std::fs::remove_file(&command).ok();
            std::os::unix::fs::symlink(binary, &command).map_err(io_error("link", &command))?;
            files.push(command);
        }

        if plan.desktop_shortcut
            && let Some(desktop) = desktop_folder()
        {
            let shortcut = desktop.join(DESKTOP_SHORTCUT);

            write_file(&shortcut, entry.as_bytes())?;
            make_executable(&shortcut)?;
            files.push(shortcut);
        }

        Ok(())
    }

    /// The desktop folder `user-dirs.dirs` names, or `~/Desktop` without that file.
    fn desktop_folder() -> Option<PathBuf> {
        dirs::desktop_dir().or_else(|| {
            dirs::home_dir()
                .map(|home| home.join("Desktop"))
                .filter(|dir| dir.is_dir())
        })
    }

    /// A freedesktop.org desktop entry; `Exec` quotes the path as the spec asks.
    pub fn desktop_entry(binary: &Path) -> String {
        let mut quoted = String::new();

        for c in binary.display().to_string().chars() {
            if matches!(c, '"' | '`' | '$' | '\\') {
                quoted.push('\\');
            }

            quoted.push(c);
        }

        format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Tinkerbench\n\
             GenericName=Warcraft III map workbench\n\
             Comment=Edit Warcraft III Reforged maps: objects, placed units, map settings and in-game interface\n\
             Exec=\"{quoted}\"\n\
             Icon={APP_IDENTIFIER}\n\
             Terminal=false\n\
             Categories=Development;Game;\n\
             StartupWMClass={BINARY}\n"
        )
    }

    /// Linux keeps no record of programs beside the setup's own.
    pub fn unregister() -> Result<(), InstallError> {
        Ok(())
    }

    /// Linux deletes a running program's file at once, so nothing waits.
    pub fn remove_after_exit(_file: &Path, _dir: &Path) {}

    /// Menus pick up new entries and icons sooner when their caches are rebuilt;
    /// both tools are optional, so a missing one is not an error.
    pub fn refresh_menus() {
        let Some(data) = dirs::data_local_dir() else {
            return;
        };

        std::process::Command::new("update-desktop-database")
            .arg(data.join("applications"))
            .status()
            .ok();
        std::process::Command::new("gtk-update-icon-cache")
            .args(["--quiet", "--ignore-theme-index"])
            .arg(data.join("icons/hicolor"))
            .status()
            .ok();
    }
}

/// The Add/Remove Programs entry, built apart from the registry so tests can
/// check it on any system.
#[cfg(any(test, windows))]
mod uninstall_entry {
    use std::path::Path;

    pub const PUBLISHER: &str = "wc3-toolkit contributors";
    pub const HOMEPAGE: &str = "https://github.com/gabaudette/wc3-toolkit";
    const BYTES_PER_KB: u64 = 1024;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Value {
        Text(String),
        Number(u32),
    }

    /// `EstimatedSize` is in kilobytes.
    pub fn values(
        dir: &Path,
        binary: &Path,
        uninstaller: &Path,
        size_in_bytes: u64,
    ) -> Vec<(&'static str, Value)> {
        let text = |value: String| Value::Text(value);
        let size = u32::try_from(size_in_bytes / BYTES_PER_KB).unwrap_or(u32::MAX);

        vec![
            ("DisplayName", text("Tinkerbench".to_owned())),
            ("DisplayVersion", text(env!("CARGO_PKG_VERSION").to_owned())),
            ("Publisher", text(PUBLISHER.to_owned())),
            ("URLInfoAbout", text(HOMEPAGE.to_owned())),
            ("DisplayIcon", text(format!("{},0", binary.display()))),
            ("InstallLocation", text(dir.display().to_string())),
            (
                "UninstallString",
                text(format!("\"{}\" --uninstall", uninstaller.display())),
            ),
            ("NoModify", Value::Number(1)),
            ("NoRepair", Value::Number(1)),
            ("EstimatedSize", Value::Number(size)),
        ]
    }
}

#[cfg(windows)]
mod windows {
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use wc3_install::APP_IDENTIFIER;

    use super::uninstall_entry::{self, Value};
    use crate::install::{InstallError, Plan, io_error};

    pub const BINARY: &str = "Tinkerbench.exe";
    const UNINSTALLER: &str = "Uninstall Tinkerbench.exe";
    const SHORTCUT: &str = "Tinkerbench.lnk";
    const START_MENU: &str = "Microsoft\\Windows\\Start Menu\\Programs";
    /// Per user, so no administrator rights are needed.
    const UNINSTALL_KEYS: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";
    /// `CREATE_NO_WINDOW`: `cmd` is a console program, and the setup has no console.
    const NO_WINDOW: u32 = 0x0800_0000;

    pub fn default_dir() -> Option<PathBuf> {
        dirs::data_local_dir().map(|dir| dir.join("Programs").join("Tinkerbench"))
    }

    pub fn make_executable(_path: &Path) -> Result<(), InstallError> {
        Ok(())
    }

    fn uninstall_key() -> String {
        format!("{UNINSTALL_KEYS}\\{APP_IDENTIFIER}")
    }

    /// Start menu and, if asked, desktop shortcuts; a copy of the setup as the
    /// uninstaller, and the Add/Remove Programs entry that runs it.
    pub fn integrate(
        plan: &Plan,
        binary: &Path,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), InstallError> {
        let start_menu = dirs::data_dir()
            .ok_or(InstallError::NoUserFolders)?
            .join(START_MENU)
            .join(SHORTCUT);

        shortcut(binary, &start_menu)?;
        files.push(start_menu);

        if plan.desktop_shortcut
            && let Some(desktop) = dirs::desktop_dir()
        {
            let path = desktop.join(SHORTCUT);

            shortcut(binary, &path)?;
            files.push(path);
        }

        let uninstaller = plan.dir.join(UNINSTALLER);

        copy_setup(&uninstaller)?;
        files.push(uninstaller.clone());

        let size = files
            .iter()
            .filter_map(|file| std::fs::metadata(file).ok())
            .map(|meta| meta.len())
            .sum();

        register(&plan.dir, binary, &uninstaller, size)
    }

    /// The running setup becomes the uninstaller, unless it already is one.
    fn copy_setup(uninstaller: &Path) -> Result<(), InstallError> {
        let setup = std::env::current_exe().map_err(io_error("find", Path::new("the setup")))?;
        let same = setup.canonicalize().ok().is_some_and(|setup| {
            uninstaller
                .canonicalize()
                .is_ok_and(|uninstaller| uninstaller == setup)
        });

        if same {
            return Ok(());
        }

        std::fs::copy(&setup, uninstaller)
            .map(drop)
            .map_err(io_error("copy the setup to", uninstaller))
    }

    fn register(
        dir: &Path,
        binary: &Path,
        uninstaller: &Path,
        size: u64,
    ) -> Result<(), InstallError> {
        let key_path = uninstall_key();
        let key = windows_registry::CURRENT_USER
            .create(&key_path)
            .map_err(registry_failure(&key_path))?;

        for (name, value) in uninstall_entry::values(dir, binary, uninstaller, size) {
            match value {
                Value::Text(text) => key.set_string(name, text),
                Value::Number(number) => key.set_u32(name, number),
            }
            .map_err(registry_failure(&key_path))?;
        }

        Ok(())
    }

    fn registry_failure<E: std::fmt::Display>(key: &str) -> impl FnOnce(E) -> InstallError + '_ {
        move |e| InstallError::Registry {
            key: key.to_owned(),
            message: e.to_string(),
        }
    }

    /// Removes the Add/Remove Programs entry; a missing one is already removed.
    pub fn unregister() -> Result<(), InstallError> {
        let key_path = uninstall_key();

        if windows_registry::CURRENT_USER.open(&key_path).is_err() {
            return Ok(());
        }

        windows_registry::CURRENT_USER
            .remove_tree(&key_path)
            .map_err(registry_failure(&key_path))
    }

    /// Windows cannot delete a running program, so the uninstaller asks `cmd`
    /// to delete it, and the then-empty folder, a moment after it exits. A
    /// failure only leaves the uninstaller behind.
    pub fn remove_after_exit(file: &Path, dir: &Path) {
        let script = format!(
            "ping -n 3 127.0.0.1 > nul & del /f /q \"{}\" & rmdir \"{}\"",
            file.display(),
            dir.display()
        );

        Command::new("cmd")
            .arg("/c")
            .raw_arg(script)
            .creation_flags(NO_WINDOW)
            .spawn()
            .ok();
    }

    fn shortcut(binary: &Path, path: &Path) -> Result<(), InstallError> {
        let failed = |e: mslnk::MSLinkError| InstallError::Shortcut {
            path: path.to_path_buf(),
            message: e.to_string(),
        };

        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(io_error("create", dir))?;
        }

        let mut link = mslnk::ShellLink::new(binary).map_err(failed)?;

        if let Some(dir) = binary.parent() {
            link.set_working_dir(Some(dir.display().to_string()));
        }

        link.create_lnk(path).map_err(failed)
    }

    pub fn refresh_menus() {}
}

#[cfg(test)]
mod entry_tests {
    use std::path::Path;

    use super::uninstall_entry::{Value, values};

    #[test]
    fn the_uninstall_entry_runs_the_uninstaller_and_states_its_size() {
        let entry = values(
            Path::new("C:\\Users\\Me\\AppData\\Local\\Programs\\Tinkerbench"),
            Path::new("C:\\Tinkerbench\\Tinkerbench.exe"),
            Path::new("C:\\Tinkerbench\\Uninstall Tinkerbench.exe"),
            3 * 1024 * 1024,
        );
        let value = |name: &str| {
            entry
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| v.clone())
        };

        assert_eq!(
            value("UninstallString"),
            Some(Value::Text(
                "\"C:\\Tinkerbench\\Uninstall Tinkerbench.exe\" --uninstall".to_owned()
            ))
        );
        assert_eq!(
            value("DisplayIcon"),
            Some(Value::Text("C:\\Tinkerbench\\Tinkerbench.exe,0".to_owned()))
        );
        assert_eq!(value("EstimatedSize"), Some(Value::Number(3072)));
        assert_eq!(
            value("DisplayVersion"),
            Some(Value::Text(env!("CARGO_PKG_VERSION").to_owned()))
        );
    }
}

#[cfg(all(test, not(windows)))]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn the_desktop_entry_quotes_its_command() {
        let entry = desktop_entry(Path::new("/home/me/My \"Apps\"/tinkerbench"));

        assert!(entry.contains("Exec=\"/home/me/My \\\"Apps\\\"/tinkerbench\"\n"));
        assert!(entry.contains("Icon=io.github.gabaudette.wc3toolkit\n"));
        assert!(entry.starts_with("[Desktop Entry]\n"));
    }
}
