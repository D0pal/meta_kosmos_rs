use std::fmt::format;

use opentelemetry::sdk::export::trace::stdout;
use tracing::Level;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_flame::FlameLayer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{fmt, prelude::*};

pub struct TraceConfig {
    pub file_name_prefix: String,
    pub dir: String,
    pub level: Level,
    pub flame: bool,
    pub console: bool,
}

pub fn init_tracing(config: TraceConfig) -> Vec<WorkerGuard> {
    let dir = config.dir;
    let file_name_prefix = config.file_name_prefix;
    let level = config.level;
    let flame = config.flame;
    let console = config.console;

    let mut guards = vec![];
    let (fmt_writter, fmt_guard) =
        tracing_appender::non_blocking(rolling::daily(&dir, &file_name_prefix));
    guards.push(fmt_guard);

    let (telemetry_writter, telemetry_guard) = tracing_appender::non_blocking(rolling::daily(
        &dir,
        format!("{}.telemetry", file_name_prefix),
    ));
    guards.push(telemetry_guard);

    let tracer = stdout::new_pipeline().with_writer(telemetry_writter).install_simple();

    let layered = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(fmt_writter)
        .with_ansi(false)
        .finish()
        .with(OpenTelemetryLayer::new(tracer));

    if (flame) {
        let (folded_writter, folded_guard) = tracing_appender::non_blocking(rolling::daily(
            &dir,
            format!("{}.folded", file_name_prefix),
        ));
        guards.push(folded_guard);

        if console {
            layered.with(fmt::Layer::default()).with(FlameLayer::new(folded_writter)).init();
        } else {
            layered.with(FlameLayer::new(folded_writter)).init()
        }
    } else {
        if console {
            layered.with(fmt::Layer::default()).init();
        } else {
            layered.init()
        }
    }

    guards
}
