//! Crash + diagnostics logging. Writes a daily-rotated file under the app's
//! log directory and mirrors to stderr in dev. Panics are captured and
//! reported via `tracing::error` with full backtrace.
//!
//! The returned `WorkerGuard` must be stored alive for the lifetime of the
//! process — drop it on app shutdown to flush pending log lines.

use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_subscriber::{fmt, EnvFilter};

/// Tracing targets for AI/voice events (E-10). Emit with
/// `tracing::info!(target: logging::AI_EVENTS_TARGET, …)` from the AI layer
/// (Épica F/G) and `VOICE_EVENTS_TARGET` from the voice layer (Épica H).
/// These are mirrored to a dedicated local file — never sent over the network.
pub const AI_EVENTS_TARGET: &str = "ai_events";
pub const VOICE_EVENTS_TARGET: &str = "voice_events";

/// Holds every non-blocking writer guard alive for the process lifetime.
/// Dropping flushes all pending log lines.
pub struct LogGuards(#[allow(dead_code)] Vec<WorkerGuard>);

pub fn init(log_dir: &Path) -> LogGuards {
    if let Err(e) = std::fs::create_dir_all(log_dir) {
        // Last-resort: stderr-only logging if we cannot create the dir.
        eprintln!("[draffity] could not create log dir {log_dir:?}: {e}");
    }

    let appender = tracing_appender::rolling::daily(log_dir, "draffity.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(appender);

    // Dedicated, local-only sink for AI/voice telemetry (E-10). Same lines
    // also land in the main log; this isolated stream makes the AI/voice
    // event audit trail easy to inspect. Zero network egress.
    let events_appender = tracing_appender::rolling::daily(log_dir, "ai-voice-events.log");
    let (events_nb, events_guard) = tracing_appender::non_blocking(events_appender);
    let events_layer = fmt::layer()
        .with_writer(events_nb)
        .with_ansi(false)
        .with_target(true)
        .with_filter(filter_fn(|meta| {
            matches!(meta.target(), AI_EVENTS_TARGET | VOICE_EVENTS_TARGET)
        }));

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,draffity_desktop_lib=debug"));

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true);

    let stderr_layer = fmt::layer().with_writer(std::io::stderr).with_target(false);

    // `try_init` is intentionally tolerant — if a global subscriber already
    // exists (e.g. set up by tests), we keep that one.
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer)
        .with(file_layer)
        .with(events_layer)
        .try_init();

    install_panic_hook();

    tracing::info!(
        log_dir = %log_dir.display(),
        version = env!("CARGO_PKG_VERSION"),
        "tracing initialised"
    );

    LogGuards(vec![guard, events_guard])
}

fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let location = info.location().map(|l| format!("{l}")).unwrap_or_default();
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.as_str()))
            .unwrap_or("<panic with non-string payload>");
        tracing::error!(location = %location, payload = %payload, "panic");
        default(info);
    }));
}
