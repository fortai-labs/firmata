use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
    prelude::*,
};

pub fn init_tracing() {
    // Get log level from environment variable or default to info
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Set up the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true)
            .with_ansi(true))
        .init();
} 