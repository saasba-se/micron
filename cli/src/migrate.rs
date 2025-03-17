use anyhow::{Error, Result};
use clap::{Arg, ArgMatches, Command};
use tokio_util::sync::CancellationToken;

use micron::{
    api::{self, AuthDuration, AuthResponse, AuthScope},
    db::Collectable,
    Database, User,
};

use crate::util::store_token;

pub fn cmd() -> Command {
    Command::new("migrate")
        .about("Perform migrations")
        .display_order(100)
}

/// Run custom migrations.
pub async fn run(matches: &ArgMatches, cancel: CancellationToken) -> Result<()> {
    todo!()
}
