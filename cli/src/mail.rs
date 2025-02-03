use std::str::FromStr;

use anyhow::Result;
use clap::{
    arg,
    builder::{PossibleValue, ValueParser},
    Arg, ArgAction, ArgMatches,
};
use tokio_util::sync::CancellationToken;

use micron::{
    auth::{hash_password, validate_password},
    email::list::Subscriber,
    Config, Database, User,
};
use uuid::Uuid;

pub fn cmd(config: &Config) -> clap::Command {
    clap::Command::new("mail")
        .subcommand_required(true)
        // .arg_required_else_help(true)
        .display_order(20)
        .about("Email-related actions")
        .subcommand(
            clap::Command::new("send")
                .arg_required_else_help(true)
                .about("Send email to subscribers")
                .arg(Arg::new("file").long("file").value_name("PATH"))
                .arg(
                    Arg::new("lists")
                        .long("lists")
                        .help("Mailing lists to send to")
                        .required(false)
                        .value_name("LISTS")
                        .value_parser(
                            config
                                .mailing
                                .lists
                                .iter()
                                .map(|list| PossibleValue::new(list))
                                .collect::<Vec<PossibleValue>>(),
                        ),
                ),
        )
        .subcommand(
            clap::Command::new("list")
                .subcommand_required(false)
                .about("Manage mailing lists")
                .arg(
                    Arg::new("name")
                        .help("Select list by name")
                        .required(false)
                        .num_args(0..=1),
                )
                .arg(
                    Arg::new("email")
                        .long("email")
                        .short('e')
                        .help("User email"),
                )
                .arg(
                    Arg::new("is_admin")
                        .short('a')
                        .long("admin")
                        .required(false)
                        .action(ArgAction::Set),
                ),
        )
}

pub async fn run(
    matches: &ArgMatches,
    remote: bool,
    config: &Config,
    cancel: CancellationToken,
) -> Result<()> {
    let db = Database::new()?;

    match matches.subcommand() {
        Some(("list", m)) => {
            match m.get_one::<String>("name") {
                Some(name) => {
                    let mut subs = db.get_collection::<Subscriber>()?;
                    subs.retain(|s| s.lists.contains(name));
                    println!("Mailing list `{}` subscribers: {:?}", name, subs);
                }

                None => {
                    // `mail list` gets a list of mailing lists

                    print!("Found mailing lists: ");
                    for (n, list) in config.mailing.lists.iter().enumerate() {
                        print!("{list}");
                        if n < config.mailing.lists.len() - 1 {
                            print!(", ");
                        }
                    }
                    print!("\n");
                }
            }
        }
        _ => unimplemented!(),
    }

    cancel.cancel();

    Ok(())
}
