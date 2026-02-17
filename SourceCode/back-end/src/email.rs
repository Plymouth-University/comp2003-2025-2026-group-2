use anyhow::{Result, anyhow};
use lettre::{
    Message, SmtpTransport, Transport, message::header::ContentType,
    transport::smtp::authentication::Credentials,
};

#[derive(Debug, Clone)]
struct SmtpConfig {
    server: String,
    username: String,
    password: String,
    from_email: String,
    from_name: String,
}

impl SmtpConfig {
    /// Loads SMTP configuration from environment variables.
    ///
    /// # Errors
    /// Returns an error if required environment variables (`SMTP_USERNAME`, `SMTP_SERVER`, etc.) are not set.
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

/// Sends a plain text email.
///
/// # Errors
/// Returns an error if SMTP is not configured, email parsing fails, or sending fails.
///
/// # Panics
/// Panics if SMTP configuration cannot be loaded.
async fn send_email(to_email: &str, subject: &str, body: &str) -> Result<()> {
    let Ok(config) = SmtpConfig::load() else {
        tracing::error!("SMTP not configured! {}", to_email);
        panic!("SMTP not configured");
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

    let creds = Credentials::new(config.username.clone(), config.password.clone());

    let (host, port) = if let Some((h, p)) = config.server.split_once(':') {
        (h.to_string(), p.parse::<u16>().unwrap_or(587))
    } else {
        (config.server.clone(), 587)
    };

    let mailer = if host == "127.0.0.1" || host == "localhost" || host == "mailhog" {
        SmtpTransport::builder_dangerous(&host)
            .credentials(creds)
            .port(port)
            .build()
    } else {
        SmtpTransport::relay(&host)
            .map_err(|e| anyhow!("Failed to connect to SMTP server: {e}"))?
            .credentials(creds)
            .build()
    };

    tokio::task::spawn_blocking(move || {
        mailer
            .send(&email)
            .map_err(|e| anyhow!("Failed to send email: {e} {config:?}"))
    })
    .await
    .map_err(|e| anyhow!("Task join error: {e}"))??;

    Ok(())
}

/// Sends an invitation email to a new user.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_invitation_email(
    to_email: &str,
    invite_link: &str,
    company_name: &str,
) -> Result<()> {
    let subject = format!("{company_name} has invited you to join LogSmart");
    let body = format!(
        "Hello,\n\n\
        You have been invited to join the company '{company_name}' on LogSmart.\n\n\
        Please click the link below to accept the invitation:\n\n\
        {invite_link}\n\n\
        This invitation link will expire in 7 days.\n\n\
        If you did not expect this invitation, you can safely ignore this email.\n\n\
        Best regards,\n\
        The LogSmart Team"
    );

    send_email(to_email, &subject, &body).await?;
    tracing::info!("Invitation email sent to {}", to_email);
    Ok(())
}

/// Sends a password reset link to a user.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_password_reset_email(to_email: &str, reset_link: &str) -> Result<()> {
    let subject = "LogSmart Password Reset Request";
    let body = format!(
        "Hello,\n\n\
        We received a request to reset your LogSmart password.\n\n\
        Please click the link below to reset your password:\n\n\
        {reset_link}\n\n\
        This link will expire in 24 hours.\n\n\
        If you did not request a password reset, please ignore this email.\n\n\
        Best regards,\n\
        The LogSmart Team"
    );

    send_email(to_email, subject, &body).await?;
    tracing::info!("Password reset email sent to {}", to_email);
    Ok(())
}

/// Sends a cancellation notice for a previously sent invitation.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_invitation_cancelled_email(to_email: &str) -> Result<()> {
    let subject = "Your LogSmart Invitation Has Been Cancelled";
    let body = "Hello,\n\n\
        We want to inform you that your invitation to join LogSmart has been cancelled by the company administrator.\n\n\
        If you believe this was done in error, please contact the company administrator directly.\n\n\
        Best regards,\n\
        The LogSmart Team".to_string();

    send_email(to_email, subject, &body).await?;
    tracing::info!("Invitation cancellation email sent to {}", to_email);
    Ok(())
}

/// Sends a branch deletion confirmation link to a user.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_branch_deletion_confirmation_email(
    to_email: &str,
    branch_name: &str,
    confirmation_link: &str,
) -> Result<()> {
    let subject = "Confirm Branch Deletion - LogSmart";
    let body = format!(
        "Hello,\n\n\
        We received a request to delete the branch '{}'.\n\n\
        This is a sensitive operation that will permanently remove the branch and disassociate any users currently assigned to it.\n\n\
        Please click the link below to confirm and proceed with the deletion:\n\n\
        {confirmation_link}\n\n\
        This link will expire in 1 hour.\n\n\
        If you did not request this deletion, please ignore this email and contact your system administrator immediately.\n\n\
        Best regards,\n\
        The LogSmart Team",
        branch_name
    );

    send_email(to_email, subject, &body).await?;
    tracing::info!("Branch deletion confirmation email sent to {}", to_email);
    Ok(())
}

/// Sends a notification that a branch has been successfully deleted.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_branch_deleted_notification_email(to_email: &str, branch_name: &str) -> Result<()> {
    let subject = "Branch Deleted - LogSmart";
    let body = format!(
        "Hello,\n\n\
        The branch '{}' has been successfully deleted from your LogSmart account.\n\n\
        Any users previously assigned to this branch have been disassociated and will need to be reassigned to other branches if needed.\n\n\
        If you have any questions, please contact your system administrator.\n\n\
        Best regards,\n\
        The LogSmart Team",
        branch_name
    );

    send_email(to_email, subject, &body).await?;
    tracing::info!("Branch deleted notification email sent to {}", to_email);
    Ok(())
}
