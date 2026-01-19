//! Client layer: orchestrates transport calls and maps transport ↔ domain.

use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::{
    ApiId, Login, Password, SendOptions, SendSms, SendSmsResponse, Status, StatusCode,
    ValidationError,
};

const DEFAULT_ENDPOINT: &str = "https://sms.ru/sms/send";

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone)]
struct HttpResponse {
    status: u16,
    body: String,
}

trait HttpTransport: Send + Sync {
    fn post_form<'a>(
        &'a self,
        url: &'a str,
        params: Vec<(String, String)>,
    ) -> BoxFuture<'a, Result<HttpResponse, Box<dyn StdError + Send + Sync>>>;
}

#[derive(Debug, Clone)]
struct ReqwestTransport {
    client: reqwest::Client,
}

impl HttpTransport for ReqwestTransport {
    fn post_form<'a>(
        &'a self,
        url: &'a str,
        params: Vec<(String, String)>,
    ) -> BoxFuture<'a, Result<HttpResponse, Box<dyn StdError + Send + Sync>>> {
        Box::pin(async move {
            let response = self.client.post(url).form(&params).send().await?;
            let status = response.status().as_u16();
            let body = response.text().await?;
            Ok(HttpResponse { status, body })
        })
    }
}

#[derive(Debug, Clone)]
pub enum Auth {
    ApiId(ApiId),
    LoginPassword { login: Login, password: Password },
}

impl Auth {
    pub fn api_id(value: impl Into<String>) -> Result<Self, ValidationError> {
        Ok(Self::ApiId(ApiId::new(value)?))
    }

    pub fn login_password(
        login: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        Ok(Self::LoginPassword {
            login: Login::new(login)?,
            password: Password::new(password)?,
        })
    }

    fn push_form_params(&self, params: &mut Vec<(String, String)>) {
        match self {
            Self::ApiId(api_id) => {
                params.push((ApiId::FIELD.to_owned(), api_id.as_str().to_owned()));
            }
            Self::LoginPassword { login, password } => {
                params.push((Login::FIELD.to_owned(), login.as_str().to_owned()));
                params.push((Password::FIELD.to_owned(), password.as_str().to_owned()));
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SmsRuError {
    #[error("transport error: {0}")]
    Transport(#[source] Box<dyn StdError + Send + Sync>),

    #[error("unexpected HTTP status: {status}")]
    HttpStatus { status: u16, body: Option<String> },

    #[error("API error: {status_code:?} {status_text:?}")]
    Api {
        status_code: StatusCode,
        status_text: Option<String>,
    },

    #[error("parse error: {0}")]
    Parse(#[source] Box<dyn StdError + Send + Sync>),

    #[error("unsupported response format: {0}")]
    UnsupportedResponseFormat(&'static str),

    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
}

#[derive(Debug, Clone)]
pub struct SmsRuClientBuilder {
    auth: Auth,
    endpoint: String,
    timeout: Option<Duration>,
    user_agent: Option<String>,
}

impl SmsRuClientBuilder {
    pub fn new(auth: Auth) -> Self {
        Self {
            auth,
            endpoint: DEFAULT_ENDPOINT.to_owned(),
            timeout: None,
            user_agent: None,
        }
    }

    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn build(self) -> Result<SmsRuClient, SmsRuError> {
        let mut builder = reqwest::Client::builder();
        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }
        if let Some(user_agent) = self.user_agent {
            builder = builder.user_agent(user_agent);
        }

        let client = builder
            .build()
            .map_err(|err| SmsRuError::Transport(Box::new(err)))?;

        Ok(SmsRuClient {
            auth: self.auth,
            endpoint: self.endpoint,
            http: Arc::new(ReqwestTransport { client }),
        })
    }
}

#[derive(Clone)]
pub struct SmsRuClient {
    auth: Auth,
    endpoint: String,
    http: Arc<dyn HttpTransport>,
}

impl SmsRuClient {
    pub fn new(auth: Auth) -> Self {
        Self {
            auth,
            endpoint: DEFAULT_ENDPOINT.to_owned(),
            http: Arc::new(ReqwestTransport {
                client: reqwest::Client::new(),
            }),
        }
    }

    pub fn builder(auth: Auth) -> SmsRuClientBuilder {
        SmsRuClientBuilder::new(auth)
    }

    pub async fn send_sms(&self, request: SendSms) -> Result<SendSmsResponse, SmsRuError> {
        if request_options(&request).json != crate::domain::JsonMode::Json {
            return Err(SmsRuError::UnsupportedResponseFormat(
                "plain-text responses are not supported; set SendOptions.json = JsonMode::Json",
            ));
        }

        let mut params = Vec::<(String, String)>::new();
        self.auth.push_form_params(&mut params);
        params.extend(crate::transport::encode_send_sms_form(&request));

        let response = self
            .http
            .post_form(&self.endpoint, params)
            .await
            .map_err(SmsRuError::Transport)?;

        if !(200..=299).contains(&response.status) {
            let body = if response.body.trim().is_empty() {
                None
            } else {
                Some(response.body)
            };
            return Err(SmsRuError::HttpStatus {
                status: response.status,
                body,
            });
        }

        let parsed = crate::transport::decode_send_sms_json_response(&request, &response.body)
            .map_err(|err| SmsRuError::Parse(Box::new(err)))?;

        if parsed.status != Status::Ok {
            return Err(SmsRuError::Api {
                status_code: parsed.status_code,
                status_text: parsed.status_text,
            });
        }

        Ok(parsed)
    }
}

fn request_options(request: &SendSms) -> &SendOptions {
    match request {
        SendSms::ToMany(to_many) => to_many.options(),
        SendSms::PerRecipient(per_recipient) => per_recipient.options(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::domain::{MessageText, RawPhoneNumber, SendOptions, SendSms, StatusCode};

    use super::*;

    #[derive(Debug, Clone)]
    struct FakeTransport {
        state: Arc<Mutex<FakeTransportState>>,
    }

    #[derive(Debug)]
    struct FakeTransportState {
        last_url: Option<String>,
        last_params: Vec<(String, String)>,
        response_status: u16,
        response_body: String,
    }

    impl FakeTransport {
        fn new(response_status: u16, response_body: impl Into<String>) -> Self {
            Self {
                state: Arc::new(Mutex::new(FakeTransportState {
                    last_url: None,
                    last_params: Vec::new(),
                    response_status,
                    response_body: response_body.into(),
                })),
            }
        }

        fn last_request(&self) -> (Option<String>, Vec<(String, String)>) {
            let state = self.state.lock().unwrap();
            (state.last_url.clone(), state.last_params.clone())
        }
    }

    impl HttpTransport for FakeTransport {
        fn post_form<'a>(
            &'a self,
            url: &'a str,
            params: Vec<(String, String)>,
        ) -> BoxFuture<'a, Result<HttpResponse, Box<dyn StdError + Send + Sync>>> {
            Box::pin(async move {
                let (status, body) = {
                    let mut state = self.state.lock().unwrap();
                    state.last_url = Some(url.to_owned());
                    state.last_params = params;
                    (state.response_status, state.response_body.clone())
                };
                Ok(HttpResponse { status, body })
            })
        }
    }

    fn assert_param(params: &[(String, String)], key: &str, value: &str) {
        assert!(
            params.iter().any(|(k, v)| k == key && v == value),
            "missing param {key}={value}; got: {params:?}"
        );
    }

    fn make_client(auth: Auth, transport: FakeTransport) -> SmsRuClient {
        SmsRuClient {
            auth,
            endpoint: "https://example.invalid/sms/send".to_owned(),
            http: Arc::new(transport),
        }
    }

    #[tokio::test]
    async fn send_sms_includes_api_id_and_parses_ok_response() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "balance": "10.00",
          "sms": {
            "79251234567": {
              "status": "OK",
              "status_code": 100,
              "sms_id": "abc123"
            }
          }
        }
        "#;

        let transport = FakeTransport::new(200, json);
        let client = make_client(Auth::api_id("test_key").unwrap(), transport.clone());

        let phone = RawPhoneNumber::new("+79251234567").unwrap();
        let request = SendSms::to_many(
            vec![phone.clone()],
            MessageText::new("hello").unwrap(),
            SendOptions::default(),
        )
        .unwrap();

        let response = client.send_sms(request).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.status_code, StatusCode::new(100));
        assert_eq!(response.balance.as_deref(), Some("10.00"));
        assert!(response.sms.contains_key(&phone));

        let (url, params) = transport.last_request();
        assert_eq!(url.as_deref(), Some("https://example.invalid/sms/send"));
        assert_param(&params, "api_id", "test_key");
        assert_param(&params, "json", "1");
        assert_param(&params, "to", "+79251234567");
        assert_param(&params, "msg", "hello");
    }

    #[tokio::test]
    async fn send_sms_includes_login_password_auth() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "sms": {}
        }
        "#;

        let transport = FakeTransport::new(200, json);
        let client = make_client(
            Auth::login_password("user", "pass").unwrap(),
            transport.clone(),
        );

        let phone = RawPhoneNumber::new("79251234567").unwrap();
        let request = SendSms::to_many(
            vec![phone],
            MessageText::new("hello").unwrap(),
            SendOptions::default(),
        )
        .unwrap();

        client.send_sms(request).await.unwrap();

        let (_, params) = transport.last_request();
        assert_param(&params, "login", "user");
        assert_param(&params, "password", "pass");
    }

    #[tokio::test]
    async fn send_sms_maps_top_level_error_to_api_error() {
        let json = r#"
        {
          "status": "ERROR",
          "status_code": 200,
          "status_text": "Invalid api_id"
        }
        "#;

        let transport = FakeTransport::new(200, json);
        let client = make_client(Auth::api_id("bad_key").unwrap(), transport);

        let phone = RawPhoneNumber::new("79251234567").unwrap();
        let request = SendSms::to_many(
            vec![phone],
            MessageText::new("hello").unwrap(),
            SendOptions::default(),
        )
        .unwrap();

        let err = client.send_sms(request).await.unwrap_err();
        match err {
            SmsRuError::Api {
                status_code,
                status_text,
            } => {
                assert_eq!(status_code.as_i32(), 200);
                assert_eq!(status_text.as_deref(), Some("Invalid api_id"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_sms_maps_non_success_http_status() {
        let transport = FakeTransport::new(500, "oops");
        let client = make_client(Auth::api_id("test_key").unwrap(), transport);

        let phone = RawPhoneNumber::new("79251234567").unwrap();
        let request = SendSms::to_many(
            vec![phone],
            MessageText::new("hello").unwrap(),
            SendOptions::default(),
        )
        .unwrap();

        let err = client.send_sms(request).await.unwrap_err();
        assert!(matches!(
            err,
            SmsRuError::HttpStatus {
                status: 500,
                body: Some(_)
            }
        ));
    }

    #[tokio::test]
    async fn send_sms_maps_empty_http_body_to_none() {
        let transport = FakeTransport::new(503, "   ");
        let client = make_client(Auth::api_id("test_key").unwrap(), transport);

        let phone = RawPhoneNumber::new("79251234567").unwrap();
        let request = SendSms::to_many(
            vec![phone],
            MessageText::new("hello").unwrap(),
            SendOptions::default(),
        )
        .unwrap();

        let err = client.send_sms(request).await.unwrap_err();
        assert!(matches!(
            err,
            SmsRuError::HttpStatus {
                status: 503,
                body: None
            }
        ));
    }

    #[tokio::test]
    async fn send_sms_rejects_plain_text_mode() {
        let transport = FakeTransport::new(200, "{}");
        let client = make_client(Auth::api_id("test_key").unwrap(), transport);

        let phone = RawPhoneNumber::new("79251234567").unwrap();
        let request = SendSms::to_many(
            vec![phone],
            MessageText::new("hello").unwrap(),
            SendOptions {
                json: crate::domain::JsonMode::Plain,
                ..Default::default()
            },
        )
        .unwrap();

        let err = client.send_sms(request).await.unwrap_err();
        assert!(matches!(err, SmsRuError::UnsupportedResponseFormat(_)));
    }

    #[tokio::test]
    async fn send_sms_maps_invalid_json_to_parse_error() {
        let transport = FakeTransport::new(200, "{ not json }");
        let client = make_client(Auth::api_id("test_key").unwrap(), transport);

        let phone = RawPhoneNumber::new("79251234567").unwrap();
        let request = SendSms::to_many(
            vec![phone],
            MessageText::new("hello").unwrap(),
            SendOptions::default(),
        )
        .unwrap();

        let err = client.send_sms(request).await.unwrap_err();
        assert!(matches!(err, SmsRuError::Parse(_)));
    }

    #[test]
    fn auth_constructors_validate_inputs() {
        assert!(Auth::api_id("   ").is_err());
        assert!(Auth::login_password("", "pass").is_err());
        assert!(Auth::login_password("user", "").is_err());
    }
}
