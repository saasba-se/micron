use anyhow::{Error, Result};
use clap::{Arg, ArgMatches, Command};
use tokio_util::sync::CancellationToken;

use micron::Database;

pub fn cmd() -> Command {
    Command::new("export")
        .about("Export information from the database")
        .display_order(70)
        .arg(
            Arg::new("collection")
                .display_order(11)
                .help("Provide collection name")
                .required(true),
        )
}

pub async fn run(matches: &ArgMatches, cancellation: CancellationToken) -> Result<()> {
    let db = Database::new()?;

    if let Some(collection) = matches.get_one::<String>("collection") {
        match collection.as_str() {
            "comments" => {
                let comment_trees = db.trees_for::<micron::Comment>()?;
                for comment_tree in comment_trees {
                    let comments = db.get_collection_at::<micron::Comment>(comment_tree)?;
                    println!("{}", serde_json::to_string_pretty(&comments)?);
                }
            }
            _ => unimplemented!(),
        }
    }

    Ok(())
}
