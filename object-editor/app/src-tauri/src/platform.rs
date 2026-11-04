//! Platform-specific setup of the app process (AGE-TECH-005: platform-specific
//! by requirement, kept in this one module). Finding the game is in `wc3-install`.

/// Keeps WebKitGTK's threads at normal priority on Linux. WebKitGTK asks
/// RealtimeKit, over the system D-Bus, to make a few of its IPC threads
/// real-time (`SCHED_RR`), which RealtimeKit grants only under a CPU limit
/// (`RLIMIT_RTTIME`, 200 ms by default): a real-time thread that stays busy that
/// long gets its whole process killed with SIGKILL. Browsing models streams
/// enough data through those threads to reach it, so the app closed without a
/// word (AGE-46). Real-time priority gains this app nothing, so its processes
/// get a system bus address that cannot exist: the request fails and the
/// threads stay normal. Nothing else here uses the system bus (file dialogs
/// and accessibility use the session bus).
#[cfg(all(unix, not(target_os = "macos")))]
#[allow(unsafe_code)]
pub fn keep_threads_at_normal_priority() {
    const SYSTEM_BUS_VAR: &str = "DBUS_SYSTEM_BUS_ADDRESS";
    const NO_SYSTEM_BUS: &str = "unix:path=/dev/null/no-system-bus";

    // SAFETY: called first in `main`, before any other thread exists, so
    // nothing reads the environment while it changes. WebKit's processes
    // inherit it when they start.
    unsafe { std::env::set_var(SYSTEM_BUS_VAR, NO_SYSTEM_BUS) };
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
pub fn keep_threads_at_normal_priority() {}
