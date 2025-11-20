use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};
use anyhow::{Result, anyhow};

pub async fn send_invitation_email(to_email: &str, invite_link: &str, company_name: &str) -> Result<()> {
    let Ok(smtp_username) = std::env::var("SMTP_USERNAME") else {
        tracing::warn!("SMTP not configured (SMTP_USERNAME missing), skipping email send to {}", to_email);
        return Ok(());
    };
    let smtp_server = std::env::var("SMTP_SERVER")
        .map_err(|_| anyhow!("SMTP_SERVER environment variable not set"))?;
    let smtp_password = std::env::var("SMTP_PASSWORD")
        .map_err(|_| anyhow!("SMTP_PASSWORD environment variable not set"))?;
    let from_email = std::env::var("SMTP_FROM_EMAIL")
        .map_err(|_| anyhow!("SMTP_FROM_EMAIL environment variable not set"))?;
    let from_name = std::env::var("SMTP_FROM_NAME")
        .unwrap_or_else(|_| "LogSmart".to_string());

    let sender = format!("{from_name} <{from_email}>");

    let email = Message::builder()
        .sender(sender.parse()
            .map_err(|e| anyhow!("Cannot parse sender address: {e}"))?)
        .reply_to(sender.parse().map_err(|e| anyhow!("Cannot parse reply-to address: {e}"))?)
        .from(sender.parse()
            .map_err(|e| anyhow!("Cannot parse from address: {e}"))?)
        .to(to_email.parse()
            .map_err(|e| anyhow!("Invalid email address: {e}"))?)
        .subject(format!("{company_name} has invited you to join LogSmart"))
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Hello,\n\n\
            You have been invited to join the company '{company_name}' on LogSmart.\n\n\
            Please click the link below to accept the invitation:\n\n\
            {invite_link}\n\n\
            This invitation link will expire in 7 days.\n\n\
            If you did not expect this invitation, you can safely ignore this email.\n\n\
            Best regards,\n\
            The LogSmart Team"
        ))
        .map_err(|e| anyhow!("Failed to build email message: {e}"))?;

    let creds = Credentials::new(smtp_username, smtp_password);

    let mailer = SmtpTransport::relay(&smtp_server)
        .map_err(|e| anyhow!("Failed to connect to SMTP server: {e}"))?
        .credentials(creds)
        .build();

    tokio::task::spawn_blocking(move || {
        mailer.send(&email)
            .map_err(|e| anyhow!("Failed to send email: {e}"))
    })
    .await
    .map_err(|e| anyhow!("Task join error: {e}"))??;

    tracing::info!("Invitation email sent to {}", to_email);
    
    Ok(())
}

pub async fn send_password_reset_email(to_email: &str, reset_link: &str) -> Result<()> {
    let Ok(smtp_username) = std::env::var("SMTP_USERNAME") else {
        tracing::warn!("SMTP not configured (SMTP_USERNAME missing), skipping email send to {}", to_email);
        return Ok(());
    };
    let smtp_server = std::env::var("SMTP_SERVER")
        .map_err(|_| anyhow!("SMTP_SERVER environment variable not set"))?;
    let smtp_password = std::env::var("SMTP_PASSWORD")
        .map_err(|_| anyhow!("SMTP_PASSWORD environment variable not set"))?;
    let from_email = std::env::var("SMTP_FROM_EMAIL")
        .map_err(|_| anyhow!("SMTP_FROM_EMAIL environment variable not set"))?;
    let from_name = std::env::var("SMTP_FROM_NAME")
        .unwrap_or_else(|_| "LogSmart".to_string());

    let sender = format!("{from_name} <{from_email}>");

    let email = Message::builder()
        .sender(sender.parse()
            .map_err(|e| anyhow!("Cannot parse sender address: {e}"))?)
        .reply_to(sender.parse().map_err(|e| anyhow!("Cannot parse reply-to address: {e}"))?)
        .from(sender.parse()
            .map_err(|e| anyhow!("Cannot parse from address: {e}"))?)
        .to(to_email.parse()
            .map_err(|e| anyhow!("Invalid email address: {e}"))?)
        .subject("LogSmart Password Reset Request")
        .header(ContentType::TEXT_PLAIN)
        .body(format!(
            "Hello,\n\n\
            We received a request to reset your LogSmart password.\n\n\
            Please click the link below to reset your password:\n\n\
            {reset_link}\n\n\
            This link will expire in 24 hours.\n\n\
            If you did not request a password reset, please ignore this email.\n\n\
            Best regards,\n\
            The LogSmart Team"
        ))
        .map_err(|e| anyhow!("Failed to build email message: {e}"))?;

    let creds = Credentials::new(smtp_username, smtp_password);

    let mailer = SmtpTransport::relay(&smtp_server)
        .map_err(|e| anyhow!("Failed to connect to SMTP server: {e}"))?
        .credentials(creds)
        .build();

    tokio::task::spawn_blocking(move || {
        mailer.send(&email)
            .map_err(|e| anyhow!("Failed to send email: {e}"))
    })
    .await
    .map_err(|e| anyhow!("Task join error: {e}"))??;

    tracing::info!("Password reset email sent to {}", to_email);
    
    Ok(())
}