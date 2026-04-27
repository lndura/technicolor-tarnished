use std::str::FromStr;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    EnvFilter, Layer, filter::Directive, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn tracing_init() -> Result<(), &'static str> {
    let Ok(notify_directive) = Directive::from_str("notify=info") else {
        return Err("Failed to set up notify directive.");
    };

    let default_directive = Into::<Directive>::into(LevelFilter::TRACE);

    let filter = EnvFilter::from_default_env()
        .add_directive(default_directive)
        .add_directive(notify_directive);

    let stdout_log = tracing_subscriber::fmt::layer()
        .with_target(true)
        .without_time()
        .with_level(true)
        .with_file(false)
        .with_line_number(false)
        .compact();

    tracing_subscriber::registry()
        .with(stdout_log.with_filter(filter))
        .init();

    Ok(())
}
