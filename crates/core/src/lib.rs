//! This library deals with our core logic, such as authorizing user
//! interactions, forwarding regular events and constructing custom requests.

pub mod config;
pub mod error;
pub mod util;

pub mod account;
pub mod profile;

use std::sync::RwLock;

use config::Config;
use email_address::EmailAddress;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};
use matrix::{
    ruma_client::{HttpClientExt, ResponseResult},
    ruma_common::api::{OutgoingRequest, SendAccessToken},
};

static COMMUNE: RwLock<Option<&'static Commune>> = RwLock::new(None);

pub struct Commune {
    pub config: Config,
    client: matrix::Client,
    // smtp: SmtpClient<TlsStream<TcpStream>>,
}

pub async fn init() {
    let mut commune = COMMUNE.write().unwrap();

    let config = Figment::new()
        .merge(Toml::file(
            Env::var("COMMUNE_CONFIG").unwrap_or("./commune-example.toml".to_owned()),
        ))
        .extract::<Config>()
        .unwrap();

    if config
        .allowed_domains
        .as_ref()
        .is_some_and(|v| !v.is_empty())
        && config
            .blocked_domains
            .as_ref()
            .is_some_and(|v| !v.is_empty())
    {
        panic!("config can only contain either allowed or blocked domains");
    }

    let client = matrix::Client::default();

    *commune = Some(Box::leak(Box::new(Commune { config, client })));
}

pub fn commune() -> &'static Commune {
    COMMUNE
        .read()
        .unwrap()
        .expect("commune should be initialized at this point")
}

impl Commune {
    pub async fn send_matrix_request<R: OutgoingRequest>(
        &self,
        request: R,
        access_token: Option<&str>,
    ) -> ResponseResult<matrix::Client, R> {
        let at = match access_token {
            Some(at) => SendAccessToken::Always(at),
            None => SendAccessToken::None,
        };

        self.client
            .send_matrix_request::<R>(self.config.matrix.host.as_str(), at, &[], request)
            .await
    }

    pub async fn send_email_verification(
        &self,
        address: EmailAddress,
        token: impl Into<String>,
    ) -> mail_send::Result<()> {
        let config = &commune().config;

        let password = config.mail.password.inner();
        let username = config
            .mail
            .username
            .as_deref()
            .unwrap_or(&password)
            .to_owned();
        let host = &config.mail.host;

        let mut smtp = SmtpClientBuilder::new(
            host.host_str()
                .expect("failed to extract host from email configuration"),
            587,
        )
        .implicit_tls(false)
        .credentials((username.as_str(), password.as_str()))
        .connect()
        .await?;

        let token = token.into();
        let from = format!("commune@{host}");
        let html = format!(
            "<p>Thanks for signing up.\n\nUse this code to finish verifying your \
             email:\n{token}</p>"
        );
        let text = format!(
            "Thanks for signing up.\n\nUse this code to finish verifying your email:\n{token}"
        );

        let message = MessageBuilder::new()
            .from(("Commune", from.as_str()))
            .to(vec![address.as_str()])
            .subject("Email Verification Code")
            .html_body(html.as_str())
            .text_body(text.as_str());

        smtp.send(message).await
    }
}
