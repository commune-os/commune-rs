use fake::{
    faker::internet::en::{FreeEmail, Password},
    Fake,
};

use reqwest::StatusCode;
use scraper::Selector;
use uuid::Uuid;

use commune::util::secret::Secret;
use commune_server::router::api::v1::account::{
    root::{AccountRegisterPayload, AccountRegisterResponse},
    verify_code::{AccountVerifyCodePayload, VerifyCodeResponse},
    verify_code_email::{AccountVerifyCodeEmailPayload, VerifyCodeEmailResponse},
};

use crate::tools::{http::HttpClient, maildev::MailDevClient};

#[tokio::test]
async fn register_account_with_success() {
    let http_client = HttpClient::new().await;
    let session = Uuid::new_v4();
    let email: String = FreeEmail().fake();
    let verify_code_pld = AccountVerifyCodePayload {
        email: email.clone(),
        session,
    };
    let verify_code_res = http_client
        .post("/api/v1/account/verify/code")
        .json(&verify_code_pld)
        .send()
        .await;
    let verify_code = verify_code_res.json::<VerifyCodeResponse>().await;

    assert!(verify_code.sent, "should return true for sent");

    let maildev = MailDevClient::new();
    let mail = maildev.latest().await.unwrap().unwrap();
    let html = mail.html();
    let code_sel = Selector::parse("#code").unwrap();
    let mut code_el = html.select(&code_sel);
    let code = code_el.next().unwrap().inner_html();
    let verify_code_email_pld = AccountVerifyCodeEmailPayload {
        email: email.clone(),
        code: Secret::new(code.clone()),
        session,
    };

    let verify_code_res = http_client
        .post("/api/v1/account/verify/code/email")
        .json(&verify_code_email_pld)
        .send()
        .await;
    let verify_code_email = verify_code_res.json::<VerifyCodeEmailResponse>().await;

    assert!(verify_code_email.valid, "should return true for valid");

    let username: String = (10..12).fake();
    let username = username.to_ascii_lowercase();
    let password: String = Password(14..20).fake();
    let request_payload = AccountRegisterPayload {
        username,
        password,
        email,
        code,
        session,
    };
    let response = http_client
        .post("/api/v1/account")
        .json(&request_payload)
        .send()
        .await;
    let response_status = response.status();
    let response_payload = response.json::<AccountRegisterResponse>().await;

    assert_eq!(
        response_status,
        StatusCode::CREATED,
        "should return 201 for created"
    );
    assert_eq!(
        request_payload.username, response_payload.credentials.username,
        "should return the same username"
    )
}
