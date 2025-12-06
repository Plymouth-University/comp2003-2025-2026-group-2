use anyhow::{Result, anyhow};
use lettre::{
    Message, SmtpTransport, Transport, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};

struct SmtpConfig {
    server: String,
    username: String,
    password: String,
    from_email: String,
    from_name: String,
}

impl SmtpConfig {
    fn load() -> Result<Self> {
        let Ok(username) = std::env::var("SMTP_USERNAME") else {
            return Err(anyhow!("SMTP_USERNAME not configured"));
        };
        let server = std::env::var("SMTP_SERVER")
            .map_err(|_| anyhow!("SMTP_SERVER environment variable not set"))?;
        let password = std::env::var("SMTP_PASSWORD")
            .map_err(|_| anyhow!("SMTP_PASSWORD environment variable not set"))?;
        let from_email = std::env::var("SMTP_FROM_EMAIL")
            .map_err(|_| anyhow!("SMTP_FROM_EMAIL environment variable not set"))?;
        let from_name = std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "LogSmart".to_string());

        Ok(SmtpConfig {
            server,
            username,
            password,
            from_email,
            from_name,
        })
    }

    fn sender_address(&self) -> String {
        format!("{} <{}>", self.from_name, self.from_email)
    }
}

async fn send_email(to_email: &str, subject: &str, body: &str) -> Result<()> {
    let config = match SmtpConfig::load() {
        Ok(cfg) => cfg,
        Err(_) => {
            tracing::warn!("SMTP not configured, skipping email send to {}", to_email);
            return Ok(());
        }
    };

    let sender = config.sender_address();

    let email = Message::builder()
        .sender(
            sender
                .parse()
                .map_err(|e| anyhow!("Cannot parse sender address: {e}"))?,
        )
        .reply_to(
            sender
                .parse()
                .map_err(|e| anyhow!("Cannot parse reply-to address: {e}"))?,
        )
        .from(
            sender
                .parse()
                .map_err(|e| anyhow!("Cannot parse from address: {e}"))?,
        )
        .to(to_email
            .parse()
            .map_err(|e| anyhow!("Invalid email address: {e}"))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| anyhow!("Failed to build email message: {e}"))?;

    let creds = Credentials::new(config.username, config.password);

    let mailer = SmtpTransport::relay(&config.server)
        .map_err(|e| anyhow!("Failed to connect to SMTP server: {e}"))?
        .credentials(creds)
        .build();

    tokio::task::spawn_blocking(move || {
        mailer
            .send(&email)
            .map_err(|e| anyhow!("Failed to send email: {e}"))
    })
    .await
    .map_err(|e| anyhow!("Task join error: {e}"))??;

    Ok(())
}

pub async fn send_invitation_email(
    to_email: &str,
    invite_link: &str,
    company_name: &str,
) -> Result<()> {
    let subject = format!("{} has invited you to join LogSmart", company_name);
    let body = format!(
        "Hello,\n\n\
        You have been invited to join the company '{}' on LogSmart.\n\n\
        Please click the link below to accept the invitation:\n\n\
        {}\n\n\
        This invitation link will expire in 7 days.\n\n\
        If you did not expect this invitation, you can safely ignore this email.\n\n\
        Best regards,\n\
        The LogSmart Team",
        company_name, invite_link
    );

    send_email(to_email, &subject, &body).await?;
    tracing::info!("Invitation email sent to {}", to_email);
    Ok(())
}

pub async fn send_password_reset_email(to_email: &str, reset_link: &str) -> Result<()> {
    let subject = "LogSmart Password Reset Request";
    let body = format!(
        "Hello,\n\n\
        We received a request to reset your LogSmart password.\n\n\
        Please click the link below to reset your password:\n\n\
        {}\n\n\
        This link will expire in 24 hours.\n\n\
        If you did not request a password reset, please ignore this email.\n\n\
        Best regards,\n\
        The LogSmart Team",
        reset_link
    );

    send_email(to_email, subject, &body).await?;
    tracing::info!("Password reset email sent to {}", to_email);
    Ok(())
}
