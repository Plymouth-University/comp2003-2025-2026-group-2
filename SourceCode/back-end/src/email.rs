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
    let config = SmtpConfig::load().map_err(|e| anyhow!("SMTP configuration error: {e}"))?;

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
        We received a request to delete the branch '{branch_name}'.\n\n\
        This is a sensitive operation that will permanently remove the branch and disassociate any users currently assigned to it.\n\n\
        Please click the link below to confirm and proceed with the deletion:\n\n\
        {confirmation_link}\n\n\
        This link will expire in 1 hour.\n\n\
        If you did not request this deletion, please ignore this email and contact your system administrator immediately.\n\n\
        Best regards,\n\
        The LogSmart Team"
    );

    send_email(to_email, subject, &body).await?;
    tracing::info!("Branch deletion confirmation email sent to {}", to_email);
    Ok(())
}

/// Sends a notification that a branch has been successfully deleted.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_branch_deleted_notification_email(
    to_email: &str,
    branch_name: &str,
) -> Result<()> {
    let subject = "Branch Deleted - LogSmart";
    let body = format!(
        "Hello,\n\n\
        The branch '{branch_name}' has been successfully deleted from your LogSmart account.\n\n\
        Any users previously assigned to this branch have been disassociated and will need to be reassigned to other branches if needed.\n\n\
        If you have any questions, please contact your system administrator.\n\n\
        Best regards,\n\
        The LogSmart Team"
    );

    send_email(to_email, subject, &body).await?;
    tracing::info!("Branch deleted notification email sent to {}", to_email);
    Ok(())
}

/// Sends company data export email.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_export_ready_notification(
    to_email: &str,
    company_name: &str,
    company_id: &str,
    filename: &str,
    frontend_url: &str,
) -> Result<()> {
    let subject = "Log Data Export Ready - LogSmart";

    let download_url =
        format!("{frontend_url}/api/companies/{company_id}/export/download/{filename}");

    let body = format!(
        "Hello,\n\n\
        Your log data export for '{company_name}' is ready to download.\n\n\
        The export contains:\n\
        - All log templates\n\
        - All submitted log entries\n\n\
        Download link: {download_url}\n\n\
        This link will be available for 7 days.\n\n\
        If you did not request this export, please contact your system administrator immediately.\n\n\
        Best regards,\n\
        The LogSmart Team"
    );

    send_email(to_email, subject, &body).await?;
    tracing::info!("Export ready notification sent to {}", to_email);
    Ok(())
}

pub async fn send_company_data_export(
    to_email: &str,
    company_name: &str,
    company_address: &str,
    export_data: &str,
) -> Result<()> {
    let subject = "Company Data Export - LogSmart";

    let data_summary = if export_data.is_empty() {
        String::new()
    } else {
        match serde_json::from_str::<serde_json::Value>(export_data) {
            Ok(data) => {
                let users_count = data["users"].as_array().map_or(0, std::vec::Vec::len);
                let branches_count = data["branches"].as_array().map_or(0, std::vec::Vec::len);
                let invitations_count =
                    data["invitations"].as_array().map_or(0, std::vec::Vec::len);
                let templates_count = data["log_templates"]
                    .as_array()
                    .map_or(0, std::vec::Vec::len);
                format!(
                    "\nExport Summary:\n\
                    - Users: {users_count}\n\
                    - Branches: {branches_count}\n\
                    - Pending Invitations: {invitations_count}\n\
                    - Log Templates: {templates_count}"
                )
            }
            Err(_) => String::new(),
        }
    };

    let body = format!(
        "Hello,\n\n\
        Your company data export for '{company_name}' is ready.\n\n\
        Company Details:\n\
        - Name: {company_name}\n\
        - Address: {company_address}\n\
        {data_summary}\n\n\
        The full export data is available via the API and was included in this notification.\n\
        Please note that this data will be retained on our servers for 30 days after the company deletion request.\n\n\
        If you did not request this export, please contact your system administrator immediately.\n\n\
        Best regards,\n\
        The LogSmart Team"
    );

    send_email(to_email, subject, &body).await?;
    tracing::info!("Company data export email sent to {}", to_email);
    Ok(())
}

/// Sends company deletion request email with confirmation link.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_company_deletion_request(
    to_email: &str,
    company_name: &str,
    company_id: &str,
    token: &str,
    frontend_url: &str,
) -> Result<()> {
    let confirm_link =
        format!("{frontend_url}/confirm-company-deletion?company_id={company_id}&token={token}");
    let subject = "Confirm Company Deletion - LogSmart";
    let body = format!(
        "Hello,\n\n\
        A request to delete '{company_name}' has been made.\n\n\
        To confirm deletion, click the link below:\n{confirm_link}\n\n\
        This link will expire in 7 days.\n\n\
        If you did not request this deletion, please ignore this email.\n\n\
        Best regards,\n\
        The LogSmart Team"
    );

    send_email(to_email, subject, &body).await?;
    tracing::info!("Company deletion request email sent to {}", to_email);
    Ok(())
}

/// Sends company deletion notification after confirmation.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_company_deleted_notification(company_name: &str) -> Result<()> {
    let support_email =
        std::env::var("SUPPORT_EMAIL").unwrap_or_else(|_| "support@logsmart.app".to_string());
    let subject = "Company Deleted - LogSmart";
    let body = format!(
        "Hello,\n\n\
        The company '{company_name}' has been deleted.\n\n\
        All data will be retained for 30 days before permanent deletion.\n\n\
        If you did not request this deletion, please contact support immediately.\n\n\
        Best regards,\n\
        The LogSmart Team"
    );

    send_email(&support_email, subject, &body).await?;
    tracing::info!("Company deletion notification sent for {}", company_name);
    Ok(())
}

/// Sends a company deletion notification to the user who confirmed the deletion.
///
/// # Errors
/// Returns an error if the email fails to send.
pub async fn send_user_company_deleted_notification(
    user_email: &str,
    company_name: &str,
) -> Result<()> {
    let subject = "Your Company Has Been Deleted - LogSmart";
    let body = format!(
        "Hello,\n\n\
        Your company '{company_name}' has been successfully deleted.\n\n\
        All data will be retained for 30 days before permanent deletion.\n\n\
        If you did not request this deletion, please contact support immediately.\n\n\
        Best regards,\n\
        The LogSmart Team"
    );

    send_email(user_email, subject, &body).await?;
    tracing::info!("Company deletion notification sent to user {}", user_email);
    Ok(())
}
