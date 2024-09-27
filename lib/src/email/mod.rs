use lettre::{
    address::AddressError, transport::smtp::authentication::Credentials, Message, SmtpTransport,
    Transport,
};

use crate::{Error, ErrorKind, Result};

fn send(message: Message, config: crate::config::Email) -> Result<()> {
    let creds = Credentials::new(config.smtp_user, config.smtp_password);

    // Open a remote connection to mail server
    let mailer = SmtpTransport::starttls_relay(&config.smtp_server)
        .map_err(|e| Error::new(ErrorKind::Other(e.to_string())))?
        .port(config.smtp_port)
        .credentials(creds)
        .build();

    // Send the email
    let response = mailer.send(&message)?;
    if response.is_positive() {
        Ok(())
    } else {
        Err(ErrorKind::EmailBadResponse(response.code().to_string()).into())
    }
}

/// Sends a verification email containing a secret key.
pub fn verification(email_addr: String, key: String, config: crate::Config) -> Result<()> {
    let message = Message::builder()
        .from(
            format!("{} <{}>", config.name, config.email.address)
                .parse()
                .unwrap(),
        )
        .reply_to(
            format!("noreply <noreply@{}>", config.domain)
                .parse()
                .unwrap(),
        )
        .to(email_addr
            .parse()
            .map_err(|e: AddressError| Error::new(ErrorKind::EmailParseError(e.to_string())))?)
        .subject("Verify email address")
        .body(format!(
            "You have created a new account at {}. Click the link below to confirm your\n\
            email address and activate your account:\n\n\
            https://{}/verify?key={}",
            config.domain, config.domain, key
        ))?;

    send(message, config.email)
}
