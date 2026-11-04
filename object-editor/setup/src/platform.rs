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

#[cfg(windows)]
mod windows {
    use std::path::{Path, PathBuf};

    use crate::install::{InstallError, Plan};

    pub const BINARY: &str = "Tinkerbench.exe";
    const SHORTCUT: &str = "Tinkerbench.lnk";
    const START_MENU: &str = "Microsoft\\Windows\\Start Menu\\Programs";

    pub fn default_dir() -> Option<PathBuf> {
        dirs::data_local_dir().map(|dir| dir.join("Programs").join("Tinkerbench"))
    }

    pub fn make_executable(_path: &Path) -> Result<(), InstallError> {
        Ok(())
    }

    /// Start menu and, if asked, desktop shortcuts.
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

        Ok(())
    }

    fn shortcut(binary: &Path, path: &Path) -> Result<(), InstallError> {
        let failed = |e: mslnk::MSLinkError| InstallError::Shortcut {
            path: path.to_path_buf(),
            message: e.to_string(),
        };

        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(crate::install::io_error("create", dir))?;
        }

        let mut link = mslnk::ShellLink::new(binary).map_err(failed)?;

        if let Some(dir) = binary.parent() {
            link.set_working_dir(Some(dir.display().to_string()));
        }

        link.create_lnk(path).map_err(failed)
    }

    pub fn refresh_menus() {}
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
