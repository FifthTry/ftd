#[derive(thiserror::Error, Debug)]
pub enum MailError {
    #[error("Mail Error: {0}")]
    Mail(#[from] lettre::error::Error),
    #[error("Address Parse Error: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("SMTP Error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
}

/// Send SMTP emails
pub struct Mailer {
    smtp_username: String,
    smtp_password: String,
    smtp_host: String,
    sender_email: String,
    sender_name: Option<String>,
}

impl Mailer {
    /// Create a new instance of Mail using values from environment variables.
    async fn from_env(ds: &fastn_ds::DocumentStore) -> Result<Self, fastn_ds::EnvironmentError> {
        let smtp_username = ds.env("FASTN_SMTP_USERNAME").await?;
        let smtp_password = ds.env("FASTN_SMTP_PASSWORD").await?;
        let smtp_host = ds.env("FASTN_SMTP_HOST").await?;
        let sender_email = ds.env("FASTN_SMTP_SENDER_EMAIL").await?;
        let sender_name = ds.env("FASTN_SMTP_SENDER_NAME").await.ok();

        Ok(Mailer {
            smtp_username,
            smtp_password,
            sender_email,
            sender_name,
            smtp_host,
        })
    }

    // TODO: add support for DKIM
    // https://en.wikipedia.org/wiki/DomainKeys_Identified_Mail
    /// send {body} as html body of the email
    /// Requires the following environment variables to be set:
    /// - FASTN_SMTP_USERNAME
    /// - FASTN_SMTP_PASSWORD
    /// - FASTN_SMTP_HOST
    /// - FASTN_SMTP_SENDER_EMAIL
    /// - FASTN_SMTP_SENDER_NAME
    pub async fn send_raw(
        enable_email: bool,
        ds: &fastn_ds::DocumentStore,
        to: lettre::message::Mailbox,
        subject: &str,
        body: String,
    ) -> Result<(), MailError> {
        println!("send_raw");
        if !enable_email {
            tracing::info!("enable_mail is not set, not sending mail to: {}", &to);
            println!("enable_mail is not set, not sending mail to: {}", &to);
            return Ok(());
        }

        // in dev mode we only log emails
        // in prod, this should fail if the env vars are not configured
        let mailer = Mailer::from_env(ds).await.expect(
            "Creating mailer requires the following environment variables: \
                \tFASTN_SMTP_USERNAME \
                \tFASTN_SMTP_PASSWORD \
                \tFASTN_SMTP_HOST \
                \tFASTN_SMTP_SENDER_EMAIL \
                \tFASTN_SMTP_SENDER_NAME",
        );

        let email = lettre::Message::builder()
            .from(lettre::message::Mailbox::new(
                mailer.sender_name.clone(),
                mailer.sender_email.parse::<lettre::Address>()?,
            ))
            .to(to)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(body)?;

        let creds = lettre::transport::smtp::authentication::Credentials::new(
            mailer.smtp_username.clone(),
            mailer.smtp_password.clone(),
        );

        let mailer = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(
            &mailer.smtp_host,
        )?
        .credentials(creds)
        .build();

        println!("mailer created");

        lettre::AsyncTransport::send(&mailer, email).await?;

        println!("mail sent");

        Ok(())
    }
}

pub enum EmailKind {
    AccountVerification,
    PasswordReset,
}

impl std::fmt::Display for EmailKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailKind::AccountVerification => write!(f, "account-verification"),
            EmailKind::PasswordReset => write!(f, "password-reset"),
        }
    }
}
