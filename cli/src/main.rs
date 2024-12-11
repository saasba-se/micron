#![allow(warnings)]

mod init;
mod login;
mod mail;
mod new;
mod status;
mod user;

mod util;

use std::time::Duration;

use clap::{Arg, ArgMatches, Command};
use saasbase::{config, Config};
use tokio_util::sync::CancellationToken;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cancel = CancellationToken::new();

    // If executed in a context where config file is available then some
    // additional context will be provided, such as available values for
    // certain subcommands' arguments. Otherwise additional context will be
    // ommited, and the config file path can still be provided through the
    // `--config` argument.
    let mut config: Config = config::load().unwrap_or_default();

    let matches = cmd(&config).get_matches();

    // Load the proper config if proper argument is provided.
    if let Some(config_path) = matches.get_one::<String>("config") {
        config = config::load_from(config_path)?;
    }

    match matches.subcommand() {
        Some(("user", m)) => user::run(m, false, cancel.clone()).await?,
        Some(("mail", m)) => mail::run(m, false, &config, cancel.clone()).await?,
        Some(("login", m)) => login::run(m, cancel.clone()).await?,
        _ => unimplemented!(),
    }

    // Wait for either ctrl_c signal or message from within server task(s)
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Initiating graceful shutdown...");
            cancel.cancel();
        },
        _ = cancel.cancelled() => {},
    }

    tokio::time::sleep(Duration::from_millis(300)).await;

    Ok(())
}

pub fn cmd(config: &Config) -> Command {
    Command::new("saasbase")
        .subcommand_required(true)
        .arg_required_else_help(true)
        // .args_conflicts_with_subcommands(true)
        // .allow_external_subcommands(true)
        .infer_subcommands(true)
        .version(VERSION)
        .author(AUTHORS)
        .about(
            "Build saas fast. Repeat.\n\
            Learn more at https://saasba.se",
        )
        .subcommand(user::cmd())
        .subcommand(mail::cmd(config))
        .subcommand(login::cmd())
        // .subcommand(ctl::user::cmd())
        .arg(Arg::new("config").value_name("PATH"))
        .arg(
            Arg::new("verbosity")
                .long("verbosity")
                .short('v')
                .display_order(100)
                .value_name("level")
                .default_value("info")
                .value_parser(["trace", "debug", "info", "warn", "error", "none"])
                .global(true)
                .help("Set the verbosity of the log output"),
        )
}
