use tracing::{info, info_span, Span};
use tracing_subscriber::field::MakeExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;

use uuid::Uuid;
use yansi::Paint;

use crate::error::{ErrorKind, Result};
use crate::Config;

#[derive(Clone, Debug, Default, Deserialize, Serialize, strum::EnumString)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[strum(serialize = "prod")]
    Production,
    #[default]
    Formatted,
    Json,
}

impl From<String> for Mode {
    fn from(input: String) -> Self {
        match input.as_str() {
            "prod" | "production" => Self::Production,
            "formatted" => Self::Formatted,
            "json" => Self::Json,
            _ => panic!("Unkown log type {}", input),
        }
    }
}

#[derive(PartialEq, Eq, Default, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
    Off,
}

impl From<&str> for Level {
    fn from(s: &str) -> Self {
        return match &*s.to_ascii_lowercase() {
            "critical" => Level::Error,
            "support" | "warn" => Level::Warn,
            "normal" => Level::Info,
            "debug" | "dbg" => Level::Debug,
            "trace" => Level::Trace,
            "off" | "none" => Level::Off,
            _ => panic!("a log level (off, trace, debug, normal, support, critical)"),
        };
    }
}

pub fn filter_layer(level: Level) -> EnvFilter {
    let filter_str = match level {
        Level::Error => "warn,rustls=off,tungstenite=off",
        Level::Warn => "warn,rustls=off,tungstenite=off",
        Level::Info => "info,rustls=off,tungstenite=off",
        Level::Debug => "debug,sled=info,tungstenite=info",
        Level::Trace => "trace,sled=info,tungstenite=info,message_io=debug,mio=debug,want=off",
        Level::Off => "off",
    };

    tracing_subscriber::filter::EnvFilter::try_new(filter_str).expect("filter string must parse")
}

#[derive(Clone)]
pub struct TracingSpan<T = Span>(T);

pub struct TracingFairing;

pub fn default_logging_layer<S>() -> impl Layer<S>
where
    S: tracing::Subscriber,
    S: for<'span> LookupSpan<'span>,
{
    let field_format = tracing_subscriber::fmt::format::debug_fn(|writer, field, value| {
        // We'll format the field name and value separated with a colon.
        if field.name() == "message" {
            write!(writer, "{:?}", Paint::new(value).bold())
        } else {
            write!(writer, "{}: {:?}", field, Paint::default(value).bold())
        }
    })
    .delimited(", ")
    .display_messages();

    tracing_subscriber::fmt::layer()
        .fmt_fields(field_format)
        // Configure the formatter to use `print!` rather than
        // `stdout().write_str(...)`, so that logs are captured by libtest's test
        // capturing.
        .with_test_writer()
}

pub fn json_logging_layer<
    S: for<'a> tracing_subscriber::registry::LookupSpan<'a> + tracing::Subscriber,
>() -> impl tracing_subscriber::Layer<S> {
    Paint::disable();

    tracing_subscriber::fmt::layer()
        .json()
        // Configure the formatter to use `print!` rather than
        // `stdout().write_str(...)`, so that logs are captured by libtest's test
        // capturing.
        .with_test_writer()
}

/// Initializes
pub fn init(config: &Config) -> Result<()> {
    use tracing_log::LogTracer;
    use tracing_subscriber::prelude::*;

    LogTracer::init().map_err(|e| ErrorKind::Other(e.to_string()))?;

    match config.tracing.mode {
        Mode::Production => {
            // loki layer
            use tracing_loki::url::Url;

            let (loki_layer, task) = tracing_loki::builder()
                .http_header("token", &config.tracing.loki_token)
                .unwrap()
                .label("host", config.address.to_string())
                .unwrap()
                .label("app", config.name.to_string())
                .unwrap()
                .build_url(
                    Url::parse(&config.tracing.loki_address)
                        .unwrap()
                        .join("/")
                        .unwrap(),
                )
                .expect("tracing_loki failed making new layer");

            // The background task needs to be spawned so the logs actually get
            // delivered to loki.
            tokio::spawn(task);

            tracing::subscriber::set_global_default(
                tracing_subscriber::registry()
                    // .with(default_logging_layer())
                    .with(loki_layer)
                    .with(json_logging_layer())
                    .with(filter_layer(config.tracing.level)),
            )
            .map_err(|e| ErrorKind::Other(e.to_string()))?;
        }
        Mode::Formatted => {
            tracing::subscriber::set_global_default(
                tracing_subscriber::registry()
                    .with(default_logging_layer())
                    .with(filter_layer(config.tracing.level)),
            )
            .map_err(|e| ErrorKind::Other(e.to_string()))?;
        }
        Mode::Json => {
            tracing::subscriber::set_global_default(
                tracing_subscriber::registry()
                    .with(json_logging_layer())
                    .with(filter_layer(config.tracing.level)),
            )
            .map_err(|e| ErrorKind::Other(e.to_string()))?;
        }
    };

    Ok(())
}
