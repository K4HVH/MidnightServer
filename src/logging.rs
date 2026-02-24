use tracing_subscriber::fmt::time::SystemTime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::config::Config;

#[derive(Debug, Clone, Copy)]
pub enum LogStyle {
    Plain,
    Compact,
    Pretty,
    Json,
}

impl LogStyle {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "plain" => Self::Plain,
            "compact" => Self::Compact,
            "pretty" => Self::Pretty,
            "json" => Self::Json,
            _ if cfg!(debug_assertions) => Self::Pretty,
            _ => Self::Plain,
        }
    }
}

pub fn init(config: &Config) {
    let style = LogStyle::from_str(&config.log_style);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match style {
        LogStyle::Plain => {
            subscriber
                .with(fmt::layer().with_ansi(false))
                .init();
        }
        LogStyle::Compact => {
            subscriber
                .with(fmt::layer().compact().with_target(false))
                .init();
        }
        LogStyle::Pretty => {
            subscriber
                .with(
                    fmt::layer()
                        .pretty()
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true)
                        .with_span_events(FmtSpan::CLOSE),
                )
                .init();
        }
        LogStyle::Json => {
            subscriber
                .with(
                    fmt::layer()
                        .json()
                        .with_current_span(true)
                        .with_span_list(true)
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_file(true)
                        .with_line_number(true)
                        .with_timer(SystemTime)
                        .flatten_event(true),
                )
                .init();
        }
    }

    tracing::info!(log_level = %config.log_level, log_style = ?style, "logging initialized");
}
