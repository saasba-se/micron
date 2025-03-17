use std::str::FromStr;

use anyhow::Result;
use clap::{arg, Arg, ArgAction, ArgMatches};
use tokio_util::sync::CancellationToken;

use micron::{
    auth::{hash_password, validate_password},
    db::Collectable,
    Database, User,
};
use uuid::Uuid;

pub fn cmd() -> clap::Command {
    clap::Command::new("user")
        .subcommand_required(true)
        // .arg_required_else_help(true)
        .display_order(10)
        .about("Inspect and manipulate users")
        .subcommand(
            clap::Command::new("add")
                .arg_required_else_help(true)
                .about("Adds new user")
                .arg(arg!(<email> "User email"))
                .arg_required_else_help(true)
                .arg(arg!(<passwd> "User password"))
                .arg_required_else_help(true)
                // .arg(arg!(-a --admin [is_admin] "User is administrator"))
                .arg(
                    Arg::new("is_admin")
                        .short('a')
                        .long("admin")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("is_disabled")
                        .short('d')
                        .long("disabled")
                        .action(ArgAction::SetTrue),
                )
                .arg(arg!(--first_name [first_name] "User first name"))
                .arg(arg!(--last_name [last_name] "User last name"))
                .arg(arg!(--full_name [full_name] "User full name"))
                .arg(arg!(--display_name [display_name] "User display name")),
        )
        .subcommand(
            clap::Command::new("get")
                .subcommand_required(false)
                .about("Gets user matching provided information")
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
                )
                .arg(arg!(-d --disabled [is_disabled] "User is disabled"))
                .arg(arg!(--first_name [first_name] "User first name"))
                .arg(arg!(--last_name [last_name] "User last name"))
                .arg(Arg::new("passwd_hash").long("passwd").short('p')),
        )
        .subcommand(
            clap::Command::new("set")
                .about("Sets user based on provided information")
                // find user based on uuid
                .arg(arg!(-e --email [email] "User email"))
                .arg(Arg::new("id"))
                .arg(Arg::new("subscription-plan").long("subscription-plan"))
                // .arg(arg!(-a --admin [is_admin] "User is administrator"))
                // .arg(arg!(-d --disabled [is_disabled] "User is disabled"))
                // .arg(arg!(--first_name [first_name] "User first name"))
                // .arg(arg!(--last_name [last_name] "User last name"))
                .arg(Arg::new("passwd_hash").long("passwd").short('p')),
        )
        .subcommand(
            clap::Command::new("rm")
                .about("Removes selected user(s)")
                // find user based on uuid
                .arg(
                    Arg::new("email")
                        .short('e')
                        .long("email")
                        .help("User email"),
                )
                .arg(Arg::new("handle").long("handle").help("User handle"))
                .arg(
                    Arg::new("all")
                        .long("all")
                        .num_args(0)
                        .help("Flag to select all users"),
                ),
        )
}

pub async fn run(sub_matches: &ArgMatches, remote: bool, cancel: CancellationToken) -> Result<()> {
    let db = Some(Database::new()?);
    let user_command = sub_matches.subcommand().unwrap_or(("get", sub_matches));
    match user_command {
        ("add", sub_matches) => {
            // email and password are always provided
            let email = sub_matches.get_one::<String>("email").cloned().unwrap();
            let passwd = sub_matches.get_one::<String>("passwd").cloned().unwrap();

            let is_admin = sub_matches.get_one::<bool>("is_admin").cloned();
            let is_disabled = sub_matches.get_one::<bool>("is_disabled").cloned();

            let name = sub_matches.get_one::<String>("name").cloned();
            let handle = sub_matches.get_one::<String>("handle").cloned();

            let user = User {
                id: Uuid::new_v4(),
                is_admin: is_admin.unwrap_or(false),
                is_disabled: is_disabled.unwrap_or(false),
                email,
                email_confirmed: false,
                password_hash: Some(hash_password(&passwd)?),
                name: name.unwrap_or("".to_string()),
                handle: handle.unwrap_or("".to_string()),
                plan: micron::user::Plan::free(),
                ..Default::default()
            };
            if remote {
                // let resp = remote_cmd(Command::AddUser(user.clone()))?;
                // if let Response::Error(e) = resp {
                //     return Err(Error::Other(format!("failed adding user (remote): {}", e)));
                // }
            } else if let Some(db) = db {
                db.set(&user)?;
            } else {
                panic!("no access to application data")
            }
            println!("Added new user {:?} ", user);
        }
        ("get", sub_matches) => {
            let email = sub_matches.get_one::<String>("email").cloned();
            let passwd_hash = sub_matches.get_one::<String>("passwd_hash").cloned();
            // let is_admin = sub_matches.get_one::<String>("is_admin").map(|s| {
            //     s.parse()
            //         .expect(&format!("can't parse arg value as boolean: {}", s))
            // });
            let first_name = sub_matches.get_one::<String>("first_name").cloned();
            let last_name = sub_matches.get_one::<String>("last_name").cloned();

            // let user_query = UserQuery {
            //     email: email.clone(),
            //     id: None,
            //     passwd_hash: passwd_hash.clone(),
            //     is_admin,
            //     first_name,
            //     last_name,
            // };

            // println!("{:?}", user_query);

            let mut found_users: Option<Vec<User>> = None;

            if remote {
                // perform remote get
                // let response = remote_cmd(Command::GetUsers(user_query))?;
                // if let Response::Users(users) = response {
                //     found_users = Some(users);
                // }
            } else if let Some(db) = db {
                // perform local get
                for user in db.get_collection::<User>()? {
                    let mut user_ok = true;
                    if let Some(email) = &email {
                        user_ok = &user.email == email;
                    }
                    if let Some(passwd_hash) = &passwd_hash {
                        if let Some(user_passwd_hash) = &user.password_hash {
                            user_ok =
                                validate_password(passwd_hash.as_bytes(), user_passwd_hash).is_ok();
                        }
                    }
                    if user_ok {
                        if let Some(ref mut found_users) = found_users {
                            found_users.push(user);
                        } else {
                            found_users = Some(vec![user])
                        }
                    }
                }
            }

            if let Some(users) = &found_users {
                println!("Found {} user(s):", users.len());
                for user in users {
                    println!("{user:?}");
                }
            } else {
                println!("Didn't find any users");
            }
        }
        ("set", sub_matches) => {
            let email = sub_matches.get_one::<String>("email").cloned();
            let id = sub_matches
                .get_one::<String>("id")
                .map(|id| Uuid::from_str(id))
                .transpose()?;

            // let user_query = UserQuery {
            //     email: email.clone(),
            //     id,
            //     passwd_hash: None,
            //     is_admin: None,
            //     first_name: None,
            //     last_name: None,
            // };

            if remote {
                // let response = remote_cmd(Command::GetUsers(user_query))?;
                // if let Response::Users(mut users) = response {
                //     let mut target_user = users
                //         .pop()
                //         .ok_or(Error::Other("failed getting user to modify".to_string()))?;
                //     if let Some(subscription_plan) =
                //         sub_matches.get_one::<String>("subscription-plan")
                //     {
                //         target_user.subscription_plan =
                //             SubscriptionPlan::from_str(subscription_plan)?;
                //     }
                //     let response = remote_cmd(Command::SetUser(target_user))?;

                //     println!("response: {:?}", response);
                // }
            } else {
                unimplemented!()
            }

            println!("Changed user");
        }
        ("rm", sub_matches) => {
            let email = sub_matches.get_one::<String>("email").cloned();
            let handle = sub_matches.get_one::<String>("handle").cloned();
            let all = sub_matches.get_one::<bool>("all").cloned();

            if remote {
                // let resp = remote_cmd(Command::AddUser(user.clone()))?;
                // if let Response::Error(e) = resp {
                //     return Err(Error::Other(format!("failed adding user (remote): {}", e)));
                // }
            } else if let Some(db) = db {
                // remove all users
                if let Some(true) = all {
                    db.clear_at(User::get_collection_name())?;
                    cancel.cancel();
                    return Ok(());
                }

                let users = db.get_collection::<User>()?;

                let user = if let Some(email) = email {
                    let users = users
                        .into_iter()
                        .filter(|u| u.email == email)
                        .collect::<Vec<_>>();
                    if users.len() > 1 {
                        return Err(anyhow::Error::msg("multiple users with that email exist"));
                    } else if users.len() == 1 {
                        db.remove(users.first().unwrap())?;
                    } else {
                        return Err(anyhow::Error::msg("no users with that email exist"));
                    }
                } else {
                    if let Some(handle) = handle {
                        let users = users
                            .into_iter()
                            .filter(|u| u.handle == handle)
                            .collect::<Vec<_>>();
                        if users.len() > 1 {
                            return Err(anyhow::Error::msg(
                                "multiple users with that handle exist",
                            ));
                        } else if users.len() == 1 {
                            db.remove(users.first().unwrap())?;
                        } else {
                            return Err(anyhow::Error::msg("no users with that handle exist"));
                        }
                    } else {
                        return Err(anyhow::Error::msg(
                            "provide either email or handle to select user to remove",
                        ));
                    }
                };
            } else {
                panic!("no access to application data")
            }

            println!("Removed user");
        }
        ("mod", sub_matches) => {
            println!("user mod");
        }
        _ => unimplemented!(),
    }

    cancel.cancel();

    Ok(())
}
