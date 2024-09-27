use anyhow::{Error, Result};
use clap::ArgMatches;
use tokio_util::sync::CancellationToken;

use saasbase::api::{self, AuthDuration, AuthResponse, AuthScope};

use crate::util::store_token;

pub fn cmd() -> clap::Command {
    use clap::{Arg, Command};
    Command::new("login")
        .about("Log in to the saasbase platform")
        .long_about(
            "Authenticate using saasbase platform credentials.\n\n\
            Provide email and password in the interactive prompt. Alternatively,\n\
            manually provide user credentials or an application token.",
        )
        .display_order(30)
        .arg(
            Arg::new("email")
                .display_order(11)
                .long("email")
                .short('e')
                .help("Provide user email")
                .required(false),
        )
        .arg(
            Arg::new("password")
                .display_order(12)
                .long("password")
                .short('p')
                .help("Provide user password")
                .required(false),
        )
        .arg(
            Arg::new("token")
                .display_order(13)
                .long("token")
                .short('t')
                .help("Provide a valid application token")
                .required(false),
        )
}

/// Authenticates user with saasbase platform credentials and stores the
/// resulting application token on the filesystem.
pub async fn login(matches: &ArgMatches, cancellation: CancellationToken) -> Result<()> {
    let mut creds = None;
    let mut token = None;
    if matches.get_one::<String>("email").is_some()
        && matches.get_one::<String>("password").is_some()
    {
        // both credentials provided, proceed
        creds = Some((
            matches.get_one::<String>("email").unwrap().to_string(),
            matches.get_one::<String>("password").unwrap().to_string(),
        ));
    } else if matches.get_one::<String>("email").is_some()
        && !matches.get_one::<String>("password").is_some()
    {
        // only email was provided, prompt for password
        let passwd = rpassword::prompt_password("Your password: ").unwrap();
        creds = Some((
            matches.get_one::<String>("email").unwrap().to_string(),
            passwd,
        ))
    } else if !matches.get_one::<String>("email").is_some()
        && matches.get_one::<String>("password").is_some()
    {
        // only password was provided, prompt for email
        let email = rpassword::prompt_password("Your email: ").unwrap();
        creds = Some((
            email,
            matches.get_one::<String>("password").unwrap().to_string(),
        ))
    } else if matches.get_one::<String>("token").is_some() {
        // token was provided
        token = Some(matches.get_one::<String>("token").unwrap().to_string());
    } else {
        // nothing was provided, prompt for credentials
        let email = rpassword::prompt_password("Your email: ").unwrap();
        let passwd = rpassword::prompt_password("Your password: ").unwrap();
        creds = Some((email, passwd))
    }

    if let Some((email, password)) = creds {
        println!(
            "attempting auth... email: {}, password: {}",
            email, password
        );
        // authenticate to get the access token
        let request = api::AuthRequest {
            email,
            password,
            scope: AuthScope::Public,
            term: AuthDuration::Long,
            // TODO add checksum to the context
            context: "bigworlds-cli".to_string(),
        };
        let response: AuthResponse = reqwest::Client::new()
            // .post("https://bigworlds.io/api/auth")
            .post("http://127.0.0.1:8000/api/auth")
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        println!("storing token: {}", response.token);
        store_token(&response.token).await?;
    } else if let Some(token) = token {
        println!("storing token: {}", token);
        store_token(&token).await?;
    }

    cancellation.cancel();
    Ok(())
}

/// Removes the previously obtained and stored application token.
pub async fn logout(matches: &ArgMatches, cancellation: CancellationToken) -> Result<()> {
    todo!()
}
