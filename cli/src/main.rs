#![allow(warnings)]

mod init;
mod login;
mod new;
mod status;

mod util;

use std::time::Duration;

use clap::{Arg, ArgMatches, Command};
use tokio_util::sync::CancellationToken;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cancel = CancellationToken::new();

    match cmd().get_matches().subcommand() {
        Some(("login", m)) => login::login(m, cancel.clone()).await?,
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

pub fn cmd() -> Command {
    Command::new("saasbase")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .version(VERSION)
        .author(AUTHORS)
        .about(
            "Build saas fast. Repeat.\n\
            Learn more at https://saasba.se",
        )
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
        .subcommand(login::cmd())
}
