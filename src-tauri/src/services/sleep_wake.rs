//! Sleep/Wake detection service for Linux/WSL2.
//!
//! Prevents the `GLib` main loop's `poll()` from blocking indefinitely after
//! a WSL2 VM pause/resume cycle. Two complementary mechanisms:
//!
//! **Approach 1 — Keepalive timeout** (always active):
//!   A periodic `glib::timeout_add_seconds` ensures the timerfd is always
//!   armed, so `poll()` never blocks for more than `KEEPALIVE_INTERVAL_SECS`.
//!
//! **Approach 2 — `D-Bus` `PrepareForSleep`** (systemd systems):
//!   Subscribes to `org.freedesktop.login1.Manager.PrepareForSleep` via
//!   `gio::DBusConnection`. The D-Bus fd is registered in `GLib`'s main
//!   context, so the signal arrival itself wakes `poll()`. On wake, we
//!   force a `gdk_display_sync()` roundtrip to re-establish the Wayland
//!   event stream.

use glib::ControlFlow;
use std::path::Path;

/// Keepalive interval in seconds.
/// Upper bound on how long `poll()` may block after VM resume.
const KEEPALIVE_INTERVAL_SECS: u32 = 5;

/// Marker for a running [`SleepWakeService`].
///
/// The `GLib` sources live for the process lifetime; no explicit cleanup needed.
pub struct SleepWakeService {
    #[allow(dead_code)]
    keepalive_source_id: glib::SourceId,
    // Hold the D-Bus connection + subscription alive.
    // Without this, the gio::DBusConnection would be dropped and the
    // signal subscription would be cancelled.
    #[allow(dead_code)]
    dbus_subscription: Option<(gio::DBusConnection, gio::SignalSubscriptionId)>,
}

/// Returns `true` if systemd-logind is likely available.
fn is_systemd_available() -> bool {
    Path::new("/run/systemd/system").exists()
}

/// Start the sleep/wake service. Must be called on the `GLib` main thread
/// (i.e., inside Tauri's `setup` closure) so that `GLib` sources are
/// registered on the correct `GMainContext`.
///
/// # Errors
///
/// Returns an error if the keepalive source cannot be registered (unlikely).
pub fn start() -> Result<SleepWakeService, String> {
    // Approach 1: always-on keepalive — prevents poll(-1) deadlock.
    // Each tick also forces a Wayland roundtrip so the compositor
    // resumes event delivery after a VM pause/resume.
    let keepalive_source_id = glib::timeout_add_seconds(
        KEEPALIVE_INTERVAL_SECS,
        || {
            force_display_sync();
            ControlFlow::Continue
        },
    );

    // Approach 2: D-Bus PrepareForSleep signal (systemd only).
    let dbus_subscription = if is_systemd_available() {
        match subscribe_prepare_for_sleep() {
            Ok(sub) => {
                eprintln!("[GoGuo] D-Bus PrepareForSleep subscribed (systemd detected)");
                Some(sub)
            }
            Err(e) => {
                eprintln!("[GoGuo] D-Bus subscription failed, keepalive-only: {e}");
                None
            }
        }
    } else {
        eprintln!(
            "[GoGuo] No systemd, using keepalive-only ({KEEPALIVE_INTERVAL_SECS}s interval)"
        );
        None
    };

    Ok(SleepWakeService {
        keepalive_source_id,
        dbus_subscription,
    })
}

/// Subscribe to `logind`'s `PrepareForSleep` signal via `GIO` D-Bus.
///
/// The `gio::DBusConnection` registers its socket fd with `GLib`'s main
/// context, so the signal arrival naturally wakes `poll()`.
fn subscribe_prepare_for_sleep() -> Result<(gio::DBusConnection, gio::SignalSubscriptionId), String>
{
    let connection = gio::bus_get_sync(gio::BusType::System, None::<&gio::Cancellable>)
        .map_err(|e| format!("bus_get_sync: {e}"))?;

    let subscription_id = connection.signal_subscribe(
        Some("org.freedesktop.login1"),
        Some("org.freedesktop.login1.Manager"),
        Some("PrepareForSleep"),
        Some("/org/freedesktop/login1"),
        None,
        gio::DBusSignalFlags::NONE,
        |_conn, _sender, _path, _iface, _signal, params| {
            let sleeping = params.child_value(0).get::<bool>().unwrap_or(true);
            if !sleeping {
                eprintln!("[GoGuo] Wake detected via D-Bus PrepareForSleep(false)");
                force_display_sync();
            }
        },
    );

    Ok((connection, subscription_id))
}

/// Force a display sync via `gdk_display_sync()`.
///
/// On Wayland this calls `wl_display_roundtrip()`.
/// On X11 this calls `XSync()`.
/// Both re-sync with the display server to restore event delivery after
/// a VM pause/resume.
fn force_display_sync() {
    if let Some(display) = gdk::Display::default() {
        display.sync();
        eprintln!("[GoGuo] Display sync completed");
    } else {
        eprintln!("[GoGuo] No GDK display available");
    }
}
