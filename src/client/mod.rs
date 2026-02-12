//! Client layer: orchestrates transport calls and maps transport ↔ domain.

use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::{
    ApiId, CheckCallAuthStatus, CheckCallAuthStatusResponse, CheckCost, CheckCostOptions,
    CheckCostResponse, CheckStatus, CheckStatusResponse, Login, Password, SendOptions, SendSms,
    SendSmsResponse, StartCallAuth, StartCallAuthResponse, Status, StatusCode, ValidationError,
};

const DEFAULT_SEND_ENDPOINT: &str = "https://sms.ru/sms/send";
const DEFAULT_COST_ENDPOINT: &str = "https://sms.ru/sms/cost";
const DEFAULT_STATUS_ENDPOINT: &str = "https://sms.ru/sms/status";
const DEFAULT_CALLCHECK_ADD_ENDPOINT: &str = "https://sms.ru/callcheck/add";
const DEFAULT_CALLCHECK_STATUS_ENDPOINT: &str = "https://sms.ru/callcheck/status";

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
/// Authentication credentials for SMS.RU API calls.
///
/// Use [`Auth::api_id`] when you have an `api_id` token, or [`Auth::login_password`]
/// if you authenticate with a login/password pair.
pub enum Auth {
    /// Authenticate via SMS.RU `api_id`.
    ApiId(ApiId),
    /// Authenticate via SMS.RU `login` + `password`.
    LoginPassword { login: Login, password: Password },
}

impl Auth {
    /// Create [`Auth::ApiId`] and validate that the value is non-empty after trimming.
    pub fn api_id(value: impl Into<String>) -> Result<Self, ValidationError> {
        Ok(Self::ApiId(ApiId::new(value)?))
    }

    /// Create [`Auth::LoginPassword`] and validate that both parts are non-empty.
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
/// Errors returned by [`SmsRuClient`].
///
/// This error preserves:
/// - HTTP-level failures (non-2xx status or transport failures),
/// - API-level failures (top-level `status != OK`),
/// - validation/parse failures.
pub enum SmsRuError {
    /// HTTP client / transport failure (DNS, TLS, timeouts, etc).
    #[error("transport error: {0}")]
    Transport(#[source] Box<dyn StdError + Send + Sync>),

    /// Non-successful HTTP status code returned by the server.
    #[error("unexpected HTTP status: {status}")]
    HttpStatus { status: u16, body: Option<String> },

    /// SMS.RU API returned an `ERROR` status with a status code/text.
    #[error("API error: {status_code:?} {status_text:?}")]
    Api {
        status_code: StatusCode,
        status_text: Option<String>,
    },

    /// Response body could not be parsed as the expected format.
    #[error("parse error: {0}")]
    Parse(#[source] Box<dyn StdError + Send + Sync>),

    /// The request asks for a response format that the client does not support.
    #[error("unsupported response format: {0}")]
    UnsupportedResponseFormat(&'static str),

    /// One of the domain constructors rejected an invalid value.
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
}

#[derive(Debug, Clone)]
/// Builder for [`SmsRuClient`].
///
/// Use this when you need to customize the endpoint, timeout, or user-agent.
pub struct SmsRuClientBuilder {
    auth: Auth,
    send_endpoint: String,
    cost_endpoint: String,
    status_endpoint: String,
    callcheck_add_endpoint: String,
    callcheck_status_endpoint: String,
    timeout: Option<Duration>,
    user_agent: Option<String>,
}

impl SmsRuClientBuilder {
    /// Create a builder with the default endpoint and no timeout/user-agent override.
    pub fn new(auth: Auth) -> Self {
        Self {
            auth,
            send_endpoint: DEFAULT_SEND_ENDPOINT.to_owned(),
            cost_endpoint: DEFAULT_COST_ENDPOINT.to_owned(),
            status_endpoint: DEFAULT_STATUS_ENDPOINT.to_owned(),
            callcheck_add_endpoint: DEFAULT_CALLCHECK_ADD_ENDPOINT.to_owned(),
            callcheck_status_endpoint: DEFAULT_CALLCHECK_STATUS_ENDPOINT.to_owned(),
            timeout: None,
            user_agent: None,
        }
    }

    /// Override all SMS.RU endpoint URLs (`sms/send`, `sms/cost`, and `sms/status`) at once.
    ///
    /// This is kept for backwards compatibility with older code that configured a
    /// single endpoint value.
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        self.send_endpoint = endpoint.clone();
        self.cost_endpoint = endpoint.clone();
        self.status_endpoint = endpoint;
        self.callcheck_add_endpoint = self.status_endpoint.clone();
        self.callcheck_status_endpoint = self.status_endpoint.clone();
        self
    }

    /// Override the SMS.RU endpoint URL for `sms/send`.
    pub fn send_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.send_endpoint = endpoint.into();
        self
    }

    /// Override the SMS.RU endpoint URL for `sms/cost`.
    pub fn cost_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.cost_endpoint = endpoint.into();
        self
    }

    /// Override the SMS.RU endpoint URL for `sms/status`.
    pub fn status_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.status_endpoint = endpoint.into();
        self
    }

    /// Override the SMS.RU endpoint URL for `callcheck/add`.
    pub fn callcheck_add_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.callcheck_add_endpoint = endpoint.into();
        self
    }

    /// Override the SMS.RU endpoint URL for `callcheck/status`.
    pub fn callcheck_status_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.callcheck_status_endpoint = endpoint.into();
        self
    }

    /// Set an HTTP client timeout applied to the entire request.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Override the HTTP `User-Agent` header.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Build a [`SmsRuClient`].
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
            send_endpoint: self.send_endpoint,
            cost_endpoint: self.cost_endpoint,
            status_endpoint: self.status_endpoint,
            callcheck_add_endpoint: self.callcheck_add_endpoint,
            callcheck_status_endpoint: self.callcheck_status_endpoint,
            http: Arc::new(ReqwestTransport { client }),
        })
    }
}

#[derive(Clone)]
/// High-level SMS.RU client.
///
/// This type orchestrates request validation, form encoding, and response parsing.
/// By default it uses:
/// - `https://sms.ru/sms/send` for sending messages
/// - `https://sms.ru/sms/cost` for checking message costs
/// - `https://sms.ru/sms/status` for checking message status
/// - `https://sms.ru/callcheck/add` for starting call authentication
/// - `https://sms.ru/callcheck/status` for checking call authentication status
///
/// All methods expect JSON responses (`json=1`).
pub struct SmsRuClient {
    auth: Auth,
    send_endpoint: String,
    cost_endpoint: String,
    status_endpoint: String,
    callcheck_add_endpoint: String,
    callcheck_status_endpoint: String,
    http: Arc<dyn HttpTransport>,
}

impl SmsRuClient {
    /// Create a client using the default endpoint.
    ///
    /// For more customization, use [`SmsRuClient::builder`].
    pub fn new(auth: Auth) -> Self {
        Self {
            auth,
            send_endpoint: DEFAULT_SEND_ENDPOINT.to_owned(),
            cost_endpoint: DEFAULT_COST_ENDPOINT.to_owned(),
            status_endpoint: DEFAULT_STATUS_ENDPOINT.to_owned(),
            callcheck_add_endpoint: DEFAULT_CALLCHECK_ADD_ENDPOINT.to_owned(),
            callcheck_status_endpoint: DEFAULT_CALLCHECK_STATUS_ENDPOINT.to_owned(),
            http: Arc::new(ReqwestTransport {
                client: reqwest::Client::new(),
            }),
        }
    }

    /// Start building a client with custom settings.
    pub fn builder(auth: Auth) -> SmsRuClientBuilder {
        SmsRuClientBuilder::new(auth)
    }

    /// Send an SMS message through SMS.RU.
    ///
    /// Constraints:
    /// - The request must have `SendOptions.json = JsonMode::Json` (plain-text responses are
    ///   currently not supported).
    ///
    /// Errors:
    /// - Returns [`SmsRuError::Validation`] for invalid domain values,
    /// - [`SmsRuError::HttpStatus`] for non-2xx HTTP responses,
    /// - [`SmsRuError::Api`] when SMS.RU returns a top-level `ERROR`.
    pub async fn send_sms(&self, request: SendSms) -> Result<SendSmsResponse, SmsRuError> {
        if send_request_options(&request).json != crate::domain::JsonMode::Json {
            return Err(SmsRuError::UnsupportedResponseFormat(
                "plain-text responses are not supported; set SendOptions.json = JsonMode::Json",
            ));
        }

        let mut params = Vec::<(String, String)>::new();
        self.auth.push_form_params(&mut params);
        params.extend(crate::transport::encode_send_sms_form(&request));

        let response = self
            .http
            .post_form(&self.send_endpoint, params)
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

    /// Check SMS cost before sending through SMS.RU.
    ///
    /// Constraints:
    /// - The request must have `CheckCostOptions.json = JsonMode::Json` (plain-text responses are
    ///   currently not supported).
    ///
    /// Errors:
    /// - Returns [`SmsRuError::Validation`] for invalid domain values,
    /// - [`SmsRuError::HttpStatus`] for non-2xx HTTP responses,
    /// - [`SmsRuError::Api`] when SMS.RU returns a top-level `ERROR`.
    pub async fn check_cost(&self, request: CheckCost) -> Result<CheckCostResponse, SmsRuError> {
        if cost_request_options(&request).json != crate::domain::JsonMode::Json {
            return Err(SmsRuError::UnsupportedResponseFormat(
                "plain-text responses are not supported; set CheckCostOptions.json = JsonMode::Json",
            ));
        }

        let mut params = Vec::<(String, String)>::new();
        self.auth.push_form_params(&mut params);
        params.extend(crate::transport::encode_check_cost_form(&request));

        let response = self
            .http
            .post_form(&self.cost_endpoint, params)
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

        let parsed = crate::transport::decode_check_cost_json_response(&request, &response.body)
            .map_err(|err| SmsRuError::Parse(Box::new(err)))?;

        if parsed.status != Status::Ok {
            return Err(SmsRuError::Api {
                status_code: parsed.status_code,
                status_text: parsed.status_text,
            });
        }

        Ok(parsed)
    }

    /// Check status for already sent SMS ids through SMS.RU.
    ///
    /// Errors:
    /// - Returns [`SmsRuError::Validation`] for invalid domain values,
    /// - [`SmsRuError::HttpStatus`] for non-2xx HTTP responses,
    /// - [`SmsRuError::Api`] when SMS.RU returns a top-level `ERROR`.
    pub async fn check_status(
        &self,
        request: CheckStatus,
    ) -> Result<CheckStatusResponse, SmsRuError> {
        let mut params = Vec::<(String, String)>::new();
        self.auth.push_form_params(&mut params);
        params.extend(crate::transport::encode_check_status_form(&request));

        let response = self
            .http
            .post_form(&self.status_endpoint, params)
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

        let parsed = crate::transport::decode_check_status_json_response(&request, &response.body)
            .map_err(|err| SmsRuError::Parse(Box::new(err)))?;

        if parsed.status != Status::Ok {
            return Err(SmsRuError::Api {
                status_code: parsed.status_code,
                status_text: parsed.status_text,
            });
        }

        Ok(parsed)
    }

    /// Start call-based phone authentication through SMS.RU.
    ///
    /// Constraints:
    /// - The request must have `StartCallAuthOptions.json = JsonMode::Json` (plain-text responses
    ///   are currently not supported).
    pub async fn start_call_auth(
        &self,
        request: StartCallAuth,
    ) -> Result<StartCallAuthResponse, SmsRuError> {
        if request.options().json != crate::domain::JsonMode::Json {
            return Err(SmsRuError::UnsupportedResponseFormat(
                "plain-text responses are not supported; set StartCallAuthOptions.json = JsonMode::Json",
            ));
        }

        let mut params = Vec::<(String, String)>::new();
        self.auth.push_form_params(&mut params);
        params.extend(crate::transport::encode_start_call_auth_form(&request));

        let response = self
            .http
            .post_form(&self.callcheck_add_endpoint, params)
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

        let parsed = crate::transport::decode_start_call_auth_json_response(&response.body)
            .map_err(|err| SmsRuError::Parse(Box::new(err)))?;

        if parsed.status != Status::Ok {
            return Err(SmsRuError::Api {
                status_code: parsed.status_code,
                status_text: parsed.status_text,
            });
        }

        Ok(parsed)
    }

    /// Check call-based phone authentication status through SMS.RU.
    ///
    /// Constraints:
    /// - The request must have `CheckCallAuthStatusOptions.json = JsonMode::Json` (plain-text
    ///   responses are currently not supported).
    pub async fn check_call_auth_status(
        &self,
        request: CheckCallAuthStatus,
    ) -> Result<CheckCallAuthStatusResponse, SmsRuError> {
        if request.options().json != crate::domain::JsonMode::Json {
            return Err(SmsRuError::UnsupportedResponseFormat(
                "plain-text responses are not supported; set CheckCallAuthStatusOptions.json = JsonMode::Json",
            ));
        }

        let mut params = Vec::<(String, String)>::new();
        self.auth.push_form_params(&mut params);
        params.extend(crate::transport::encode_check_call_auth_status_form(
            &request,
        ));

        let response = self
            .http
            .post_form(&self.callcheck_status_endpoint, params)
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

        let parsed = crate::transport::decode_check_call_auth_status_json_response(&response.body)
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

fn send_request_options(request: &SendSms) -> &SendOptions {
    match request {
        SendSms::ToMany(to_many) => to_many.options(),
        SendSms::PerRecipient(per_recipient) => per_recipient.options(),
    }
}

fn cost_request_options(request: &CheckCost) -> &CheckCostOptions {
    match request {
        CheckCost::ToMany(to_many) => to_many.options(),
        CheckCost::PerRecipient(per_recipient) => per_recipient.options(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::domain::{
        CallCheckId, CheckCallAuthStatus, CheckCallAuthStatusOptions, CheckCost, CheckCostOptions,
        CheckStatus, MessageText, RawPhoneNumber, SendOptions, SendSms, SmsId, StartCallAuth,
        StartCallAuthOptions, StatusCode,
    };

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
            send_endpoint: "https://example.invalid/sms/send".to_owned(),
            cost_endpoint: "https://example.invalid/sms/cost".to_owned(),
            status_endpoint: "https://example.invalid/sms/status".to_owned(),
            callcheck_add_endpoint: "https://example.invalid/callcheck/add".to_owned(),
            callcheck_status_endpoint: "https://example.invalid/callcheck/status".to_owned(),
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

    #[tokio::test]
    async fn check_cost_uses_cost_endpoint_and_parses_ok_response() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "total_cost": 0.50,
          "total_sms": 1,
          "sms": {
            "79251234567": {
              "status": "OK",
              "status_code": 100,
              "cost": 0.50,
              "sms": 1
            }
          }
        }
        "#;
        let transport = FakeTransport::new(200, json);
        let client = make_client(Auth::api_id("test_key").unwrap(), transport.clone());
        let phone = RawPhoneNumber::new("+79251234567").unwrap();
        let request = CheckCost::to_many(
            vec![phone.clone()],
            MessageText::new("hello").unwrap(),
            CheckCostOptions::default(),
        )
        .unwrap();

        let response = client.check_cost(request).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.status_code, StatusCode::new(100));
        assert_eq!(response.total_cost.as_deref(), Some("0.50"));
        assert_eq!(response.total_sms, Some(1));
        assert_eq!(
            response.sms.get(&phone).and_then(|it| it.cost.as_deref()),
            Some("0.50")
        );

        let (url, params) = transport.last_request();
        assert_eq!(url.as_deref(), Some("https://example.invalid/sms/cost"));
        assert_param(&params, "api_id", "test_key");
        assert_param(&params, "json", "1");
        assert_param(&params, "to", "+79251234567");
        assert_param(&params, "msg", "hello");
    }

    #[tokio::test]
    async fn check_cost_maps_top_level_error_to_api_error() {
        let json = r#"
        {
          "status": "ERROR",
          "status_code": 200,
          "status_text": "Invalid api_id"
        }
        "#;

        let transport = FakeTransport::new(200, json);
        let client = make_client(Auth::api_id("bad_key").unwrap(), transport);
        let request = CheckCost::to_many(
            vec![RawPhoneNumber::new("79251234567").unwrap()],
            MessageText::new("hello").unwrap(),
            CheckCostOptions::default(),
        )
        .unwrap();

        let err = client.check_cost(request).await.unwrap_err();
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
    async fn check_cost_maps_non_success_http_status() {
        let transport = FakeTransport::new(503, "oops");
        let client = make_client(Auth::api_id("test_key").unwrap(), transport);
        let request = CheckCost::to_many(
            vec![RawPhoneNumber::new("79251234567").unwrap()],
            MessageText::new("hello").unwrap(),
            CheckCostOptions::default(),
        )
        .unwrap();

        let err = client.check_cost(request).await.unwrap_err();
        assert!(matches!(
            err,
            SmsRuError::HttpStatus {
                status: 503,
                body: Some(_)
            }
        ));
    }

    #[tokio::test]
    async fn check_cost_rejects_plain_text_mode() {
        let transport = FakeTransport::new(200, "{}");
        let client = make_client(Auth::api_id("test_key").unwrap(), transport);

        let request = CheckCost::to_many(
            vec![RawPhoneNumber::new("79251234567").unwrap()],
            MessageText::new("hello").unwrap(),
            CheckCostOptions {
                json: crate::domain::JsonMode::Plain,
                ..Default::default()
            },
        )
        .unwrap();

        let err = client.check_cost(request).await.unwrap_err();
        assert!(matches!(err, SmsRuError::UnsupportedResponseFormat(_)));
    }

    #[test]
    fn auth_constructors_validate_inputs() {
        assert!(Auth::api_id("   ").is_err());
        assert!(Auth::login_password("", "pass").is_err());
        assert!(Auth::login_password("user", "").is_err());
    }

    #[tokio::test]
    async fn check_status_uses_status_endpoint_and_parses_ok_response() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "balance": 10.00,
          "sms": {
            "000000-000001": {
              "status": "OK",
              "status_code": 103,
              "cost": 0.50
            }
          }
        }
        "#;
        let transport = FakeTransport::new(200, json);
        let client = make_client(Auth::api_id("test_key").unwrap(), transport.clone());
        let id = SmsId::new("000000-000001").unwrap();
        let request = CheckStatus::one(id.clone());

        let response = client.check_status(request).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.status_code, StatusCode::new(100));
        assert_eq!(response.balance.as_deref(), Some("10.00"));
        assert_eq!(
            response.sms.get(&id).and_then(|it| it.cost.as_deref()),
            Some("0.50")
        );

        let (url, params) = transport.last_request();
        assert_eq!(url.as_deref(), Some("https://example.invalid/sms/status"));
        assert_param(&params, "api_id", "test_key");
        assert_param(&params, "json", "1");
        assert_param(&params, "sms_id", "000000-000001");
    }

    #[tokio::test]
    async fn check_status_maps_top_level_error_to_api_error() {
        let json = r#"
        {
          "status": "ERROR",
          "status_code": 200,
          "status_text": "Invalid api_id"
        }
        "#;

        let transport = FakeTransport::new(200, json);
        let client = make_client(Auth::api_id("bad_key").unwrap(), transport);
        let request = CheckStatus::one(SmsId::new("000000-000001").unwrap());

        let err = client.check_status(request).await.unwrap_err();
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
    async fn check_status_maps_non_success_http_status() {
        let transport = FakeTransport::new(503, "oops");
        let client = make_client(Auth::api_id("test_key").unwrap(), transport);
        let request = CheckStatus::one(SmsId::new("000000-000001").unwrap());

        let err = client.check_status(request).await.unwrap_err();
        assert!(matches!(
            err,
            SmsRuError::HttpStatus {
                status: 503,
                body: Some(_)
            }
        ));
    }

    #[tokio::test]
    async fn start_call_auth_uses_endpoint_and_parses_ok_response() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "check_id": "201737-542",
          "call_phone": "78005008275"
        }
        "#;
        let transport = FakeTransport::new(200, json);
        let client = make_client(Auth::api_id("test_key").unwrap(), transport.clone());
        let request = StartCallAuth::new(
            RawPhoneNumber::new("79251234567").unwrap(),
            StartCallAuthOptions::default(),
        );

        let response = client.start_call_auth(request).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.status_code, StatusCode::new(100));
        assert_eq!(
            response.check_id.as_ref().map(CallCheckId::as_str),
            Some("201737-542")
        );
        assert_eq!(
            response.call_phone.as_ref().map(RawPhoneNumber::raw),
            Some("78005008275")
        );

        let (url, params) = transport.last_request();
        assert_eq!(
            url.as_deref(),
            Some("https://example.invalid/callcheck/add")
        );
        assert_param(&params, "api_id", "test_key");
        assert_param(&params, "json", "1");
        assert_param(&params, "phone", "79251234567");
    }

    #[tokio::test]
    async fn start_call_auth_rejects_plain_text_mode() {
        let transport = FakeTransport::new(200, "{}");
        let client = make_client(Auth::api_id("test_key").unwrap(), transport);
        let request = StartCallAuth::new(
            RawPhoneNumber::new("79251234567").unwrap(),
            StartCallAuthOptions {
                json: crate::domain::JsonMode::Plain,
            },
        );

        let err = client.start_call_auth(request).await.unwrap_err();
        assert!(matches!(err, SmsRuError::UnsupportedResponseFormat(_)));
    }

    #[tokio::test]
    async fn check_call_auth_status_uses_endpoint_and_parses_ok_response() {
        let json = r#"
        {
          "status": "OK",
          "status_code": 100,
          "check_status": "401",
          "check_status_text": "confirmed"
        }
        "#;
        let transport = FakeTransport::new(200, json);
        let client = make_client(Auth::api_id("test_key").unwrap(), transport.clone());
        let request = CheckCallAuthStatus::new(
            CallCheckId::new("201737-542").unwrap(),
            CheckCallAuthStatusOptions::default(),
        );

        let response = client.check_call_auth_status(request).await.unwrap();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.status_code, StatusCode::new(100));
        assert_eq!(response.check_status.map(|code| code.as_i32()), Some(401));

        let (url, params) = transport.last_request();
        assert_eq!(
            url.as_deref(),
            Some("https://example.invalid/callcheck/status")
        );
        assert_param(&params, "api_id", "test_key");
        assert_param(&params, "json", "1");
        assert_param(&params, "check_id", "201737-542");
    }

    #[tokio::test]
    async fn check_call_auth_status_rejects_plain_text_mode() {
        let transport = FakeTransport::new(200, "{}");
        let client = make_client(Auth::api_id("test_key").unwrap(), transport);
        let request = CheckCallAuthStatus::new(
            CallCheckId::new("201737-542").unwrap(),
            CheckCallAuthStatusOptions {
                json: crate::domain::JsonMode::Plain,
            },
        );

        let err = client.check_call_auth_status(request).await.unwrap_err();
        assert!(matches!(err, SmsRuError::UnsupportedResponseFormat(_)));
    }

    #[test]
    fn builder_endpoint_overrides_are_applied() {
        let client = SmsRuClient::builder(Auth::api_id("key").unwrap())
            .endpoint("https://example.invalid/all")
            .build()
            .unwrap();
        assert_eq!(client.send_endpoint, "https://example.invalid/all");
        assert_eq!(client.cost_endpoint, "https://example.invalid/all");
        assert_eq!(client.status_endpoint, "https://example.invalid/all");
        assert_eq!(client.callcheck_add_endpoint, "https://example.invalid/all");
        assert_eq!(
            client.callcheck_status_endpoint,
            "https://example.invalid/all"
        );

        let client = SmsRuClient::builder(Auth::api_id("key").unwrap())
            .send_endpoint("https://example.invalid/sms/send")
            .cost_endpoint("https://example.invalid/sms/cost")
            .status_endpoint("https://example.invalid/sms/status")
            .callcheck_add_endpoint("https://example.invalid/callcheck/add")
            .callcheck_status_endpoint("https://example.invalid/callcheck/status")
            .build()
            .unwrap();
        assert_eq!(client.send_endpoint, "https://example.invalid/sms/send");
        assert_eq!(client.cost_endpoint, "https://example.invalid/sms/cost");
        assert_eq!(client.status_endpoint, "https://example.invalid/sms/status");
        assert_eq!(
            client.callcheck_add_endpoint,
            "https://example.invalid/callcheck/add"
        );
        assert_eq!(
            client.callcheck_status_endpoint,
            "https://example.invalid/callcheck/status"
        );
    }
}
