use futures::TryFutureExt;
use lettre::{
    address::AddressError,
    message::{MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, Transport,
};

use crate::{Error, ErrorKind, Result};

pub mod list;

pub async fn send_async(message: Message, config: crate::config::Email) -> Result<()> {
    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_password.clone());

    // Open a remote connection to mail server
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_server)
        .map_err(|e| Error::new(ErrorKind::Other(e.to_string())))?
        .port(config.smtp_port)
        .credentials(creds)
        .build();

    // Send the email
    let response = mailer.send(message).await?;
    if response.is_positive() {
        Ok(())
    } else {
        Err(ErrorKind::EmailBadResponse(response.code().to_string()).into())
    }
}

/// Sends an email message containing a link used for email confirmation.
pub fn confirmation(email_addr: String, key: String, config: &crate::Config) -> Result<()> {
    let (subject, plain_body, html_body) = config
        .email
        .confirmation
        .clone()
        .map(|(s, plain, html)| (s, plain.replace("{key}", &key), html.replace("{key}", &key)))
        .unwrap_or((
            format!("Welcome to {}", config.domain),
            format!(
                "You have created a new account at {}. Click the link below to confirm your\n\
            email address and activate your account:\n\n\
            https://{}/confirm/{}",
                config.domain, config.domain, key
            ),
            format!("unimplemented"),
        ));

    let message = Message::builder()
        .from(
            format!("{} <{}>", config.name, config.email.address)
                .parse()
                .map_err(|e: AddressError| Error::new(ErrorKind::EmailParseError(e.to_string())))?,
        )
        .reply_to(
            format!("noreply <noreply@{}>", config.domain)
                .parse()
                .map_err(|e: AddressError| Error::new(ErrorKind::EmailParseError(e.to_string())))?,
        )
        .to(email_addr
            .parse()
            .map_err(|e: AddressError| Error::new(ErrorKind::EmailParseError(e.to_string())))?)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(plain_body))
                .singlepart(SinglePart::html(html_body)),
        )?;

    let email_config = config.email.clone();
    tokio::spawn(async move {
        if let Err(e) = send_async(message, email_config).await {
            log::error!("{e}")
        }
    });

    Ok(())
}

/// Sends an email message containing a link used for email confirmation.
pub fn mailing_confirmation(email_addr: String, key: String, config: &crate::Config) -> Result<()> {
    let (subject, plain_body, html_body) = config
        .email
        .mailing_confirmation
        .clone()
        .map(|(s, plain, html)| (s, plain.replace("{key}", &key), html.replace("{key}", &key)))
        .unwrap_or((
            format!(
                "Hi! Please confirm your {} newsletter subscription",
                config.domain
            ),
            format!(
                "Looks like you have requested to receive an occasional email from us.\n\
                Click the link below to confirm your email address and activate your subscription:\n\n\
            https://{}/mailing/confirm/{}",
                config.domain, key
            ),
            format!("unimplemented"),
        ));

    let message = Message::builder()
        .from(
            format!("{} <{}>", config.name, config.email.address)
                .parse()
                .map_err(|e: AddressError| Error::new(ErrorKind::EmailParseError(e.to_string())))?,
        )
        .reply_to(
            format!("noreply <noreply@{}>", config.domain)
                .parse()
                .map_err(|e: AddressError| Error::new(ErrorKind::EmailParseError(e.to_string())))?,
        )
        .to(email_addr
            .parse()
            .map_err(|e: AddressError| Error::new(ErrorKind::EmailParseError(e.to_string())))?)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(plain_body))
                .singlepart(SinglePart::html(html_body)),
        )?;

    let email_config = config.email.clone();
    tokio::spawn(async move {
        if let Err(e) = send_async(message, email_config).await {
            log::error!("{e}")
        }
    });

    Ok(())
}
