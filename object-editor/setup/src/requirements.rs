//! What Tinkerbench needs from the system before it can start: WebKitGTK on
//! Linux, the WebView2 runtime on Windows.

use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Requirements {
    Ready,
    /// What is missing, and the command that installs it when the system's
    /// package manager is known.
    Missing {
        missing: Vec<String>,
        fix: Option<Fix>,
    },
    /// The check itself could not run.
    Unknown(String),
}

/// A command run as administrator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub summary: String,
    pub command: Vec<String>,
}

impl Fix {
    /// The command for a terminal, for when the setup cannot run it.
    pub fn for_terminal(&self) -> String {
        format!("sudo {}", self.command.join(" "))
    }

    /// Asks for the administrator password in a system dialog (`pkexec`), then
    /// runs the command. In a dry run, only says what it would run.
    pub fn apply(&self, dry_run: bool) -> Result<String, String> {
        if dry_run {
            return Ok(format!(
                "Dry run: would run pkexec {}",
                self.command.join(" ")
            ));
        }

        let output = Command::new("pkexec")
            .args(&self.command)
            .output()
            .map_err(|e| {
                format!(
                    "Could not ask for the administrator password ({e}). Run this in a terminal instead: {}",
                    self.for_terminal()
                )
            })?;
        let log = String::from_utf8_lossy(&output.stdout).into_owned()
            + &String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(log)
        } else {
            Err(format!(
                "The install did not finish ({}). {}",
                output.status,
                log.lines().last().unwrap_or_default()
            ))
        }
    }
}

#[cfg(not(windows))]
pub use unix::check;

#[cfg(windows)]
pub use windows::check;

#[cfg(any(test, not(windows)))]
mod linux_packages {
    use super::Fix;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Family {
        Debian,
        Fedora,
        Arch,
        Suse,
    }

    /// The distribution family from `/etc/os-release`'s `ID` and `ID_LIKE`.
    pub fn family(os_release: &str) -> Option<Family> {
        let ids: Vec<String> = os_release
            .lines()
            .filter_map(|line| {
                line.strip_prefix("ID=")
                    .or_else(|| line.strip_prefix("ID_LIKE="))
            })
            .flat_map(|value| {
                value
                    .trim_matches(|c| c == '"' || c == '\'')
                    .split_whitespace()
                    .map(str::to_ascii_lowercase)
                    .collect::<Vec<_>>()
            })
            .collect();

        ids.iter().find_map(|id| match id.as_str() {
            "debian" | "ubuntu" => Some(Family::Debian),
            "fedora" | "rhel" | "centos" => Some(Family::Fedora),
            "arch" => Some(Family::Arch),
            id if id.contains("suse") => Some(Family::Suse),
            _ => None,
        })
    }

    /// WebKitGTK's package; its dependencies bring GTK, libsoup and the rest.
    pub fn webkit_fix(family: Family) -> Fix {
        let (summary, command): (&str, &[&str]) = match family {
            Family::Debian => (
                "WebKitGTK (libwebkit2gtk-4.1-0) through apt",
                &[
                    "sh",
                    "-c",
                    "apt-get update && apt-get install -y libwebkit2gtk-4.1-0",
                ],
            ),
            Family::Fedora => (
                "WebKitGTK (webkit2gtk4.1) through dnf",
                &["dnf", "install", "-y", "webkit2gtk4.1"],
            ),
            Family::Arch => (
                "WebKitGTK (webkit2gtk-4.1) through pacman",
                &["pacman", "-S", "--needed", "--noconfirm", "webkit2gtk-4.1"],
            ),
            Family::Suse => (
                "WebKitGTK (libwebkit2gtk-4_1-0) through zypper",
                &[
                    "zypper",
                    "--non-interactive",
                    "install",
                    "libwebkit2gtk-4_1-0",
                ],
            ),
        };

        Fix {
            summary: summary.to_owned(),
            command: command.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    /// The libraries `ldd` reports as `not found`.
    pub fn missing_libraries(ldd_output: &str) -> Vec<String> {
        ldd_output
            .lines()
            .filter(|line| line.contains("=> not found"))
            .filter_map(|line| line.split_whitespace().next())
            .map(str::to_owned)
            .collect()
    }
}

#[cfg(not(windows))]
mod unix {
    use std::io::Write;
    use std::process::Command;

    use super::Requirements;
    use super::linux_packages::{family, missing_libraries, webkit_fix};
    use crate::payload;

    const OS_RELEASE: &str = "/etc/os-release";

    /// Asks the system's loader which of Tinkerbench's libraries it cannot find.
    pub fn check() -> Requirements {
        if !payload::has_app() {
            return Requirements::Unknown(
                "This copy of the setup does not contain Tinkerbench, so there is nothing to check."
                    .to_owned(),
            );
        }

        match missing() {
            Ok(missing) if missing.is_empty() => Requirements::Ready,
            Ok(missing) => {
                let fix = std::fs::read_to_string(OS_RELEASE)
                    .ok()
                    .and_then(|text| family(&text))
                    .map(webkit_fix);

                Requirements::Missing { missing, fix }
            }
            Err(e) => Requirements::Unknown(e),
        }
    }

    /// `/tmp` can forbid running programs, which `ldd` needs, so the copy goes
    /// in the user's cache folder.
    fn missing() -> Result<Vec<String>, String> {
        let binary =
            payload::app_binary().map_err(|e| format!("Cannot unpack Tinkerbench: {e}"))?;
        let dir = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);

        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;

        let mut copy = tempfile::Builder::new()
            .prefix("tinkerbench-check")
            .tempfile_in(&dir)
            .map_err(|e| format!("Cannot write in {}: {e}", dir.display()))?;

        copy.write_all(&binary)
            .map_err(|e| format!("Cannot write the copy to check: {e}"))?;
        crate::platform::make_executable(copy.path()).map_err(|e| e.to_string())?;

        let output = Command::new("ldd")
            .arg(copy.path())
            .output()
            .map_err(|e| format!("Cannot run ldd to check the system libraries: {e}"))?;

        Ok(missing_libraries(&String::from_utf8_lossy(&output.stdout)))
    }
}

#[cfg(windows)]
mod windows {
    use super::Requirements;

    /// The WebView2 runtime's client key; `pv` holds its version once installed.
    const WEBVIEW2_KEYS: &[&str] = &[
        r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
    ];
    const VERSION_VALUE: &str = "pv";
    const NO_VERSION: &str = "0.0.0.0";

    pub fn check() -> Requirements {
        let installed = [
            windows_registry::LOCAL_MACHINE,
            windows_registry::CURRENT_USER,
        ]
        .into_iter()
        .flat_map(|hive| {
            WEBVIEW2_KEYS
                .iter()
                .filter_map(move |key| hive.open(key).ok())
        })
        .filter_map(|key| key.get_string(VERSION_VALUE).ok())
        .any(|version| !version.is_empty() && version != NO_VERSION);

        if installed {
            Requirements::Ready
        } else {
            Requirements::Missing {
                missing: vec!["Microsoft Edge WebView2 Runtime".to_owned()],
                fix: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::linux_packages::*;

    #[test]
    fn os_release_names_the_package_manager() {
        assert_eq!(
            family("NAME=\"Linux Mint\"\nID=linuxmint\nID_LIKE=\"ubuntu debian\"\n"),
            Some(Family::Debian)
        );
        assert_eq!(family("ID=fedora\n"), Some(Family::Fedora));
        assert_eq!(family("ID=endeavouros\nID_LIKE=arch\n"), Some(Family::Arch));
        assert_eq!(
            family("ID=\"opensuse-tumbleweed\"\nID_LIKE=\"opensuse suse\"\n"),
            Some(Family::Suse)
        );
        assert_eq!(family("ID=nixos\n"), None);
    }

    #[test]
    fn ldd_reports_what_is_missing() {
        let output = "\tlinux-vdso.so.1 (0x00007ffd)\n\
            \tlibwebkit2gtk-4.1.so.0 => not found\n\
            \tlibgtk-3.so.0 => /lib/x86_64-linux-gnu/libgtk-3.so.0 (0x00007f)\n\
            \tlibsoup-3.0.so.0 => not found\n";

        assert_eq!(
            missing_libraries(output),
            vec!["libwebkit2gtk-4.1.so.0", "libsoup-3.0.so.0"]
        );
    }

    #[test]
    fn fixes_install_webkit_with_the_right_tool() {
        let fix = webkit_fix(Family::Debian);

        assert_eq!(fix.command[0], "sh");
        assert!(fix.command[2].contains("apt-get install -y libwebkit2gtk-4.1-0"));
        assert_eq!(
            webkit_fix(Family::Arch).for_terminal(),
            "sudo pacman -S --needed --noconfirm webkit2gtk-4.1"
        );
    }
}
