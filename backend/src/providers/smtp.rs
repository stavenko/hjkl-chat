use lettre::message::{Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport};
use async_trait::async_trait;

#[derive(thiserror::Error, Debug)]
pub enum SMTPProviderError {
    #[error("SMTP error: {0}")]
    Smtp(#[from] lettre::error::Error),
    #[error("SMTP transport error: {0}")]
    SmtpTransport(#[from] lettre::transport::smtp::Error),
    #[error("Address error: {0}")]
    Address(#[from] lettre::address::AddressError),
}

pub type SMTPProviderResult<T> = Result<T, SMTPProviderError>;

#[async_trait]
pub trait SmtpClient: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> SMTPProviderResult<()>;
}

pub struct SMTPProvider {
    transporter: AsyncSmtpTransport<lettre::Tokio1Executor>,
    from_address: Mailbox,
}

#[async_trait]
impl SmtpClient for SMTPProvider {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> SMTPProviderResult<()> {
        let to_address = to.parse::<Mailbox>()?;

        let email = Message::builder()
            .from(self.from_address.clone())
            .to(to_address)
            .subject(subject)
            .body(body.to_string())?;

        self.transporter.send(email).await.map_err(SMTPProviderError::SmtpTransport)?;
        Ok(())
    }
}

#[allow(dead_code)]
impl SMTPProvider {
    pub fn new(
        host: &str,
        port: u16,
        _use_tls: bool,
        username: &str,
        password: &str,
        from_email: &str,
    ) -> SMTPProviderResult<Self> {
        let creds = Credentials::new(username.to_string(), password.to_string());

        let transporter = AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous(host)
            .port(port)
            .credentials(creds)
            .build();

        let from_address = from_email.parse::<Mailbox>()?;

        Ok(SMTPProvider {
            transporter,
            from_address,
        })
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> SMTPProviderResult<()> {
        let to_address = to.parse::<Mailbox>()?;

        let email = Message::builder()
            .from(self.from_address.clone())
            .to(to_address)
            .subject(subject)
            .body(body.to_string())?;

        self.transporter.send(email).await.map_err(SMTPProviderError::SmtpTransport)?;
        Ok(())
    }
}