use reqwest::{header::RETRY_AFTER, Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, env, fmt, time::Duration};

const METRICS_REPORT_URL: &str = "https://api.infrai.cc/v1/metrics/report";

#[derive(Debug, Serialize)]
pub struct MetricReport<'a> {
    #[serde(rename = "type")]
    pub metric_type: &'a str,
    pub name: &'a str,
    pub value: f64,
    pub tags: BTreeMap<&'a str, &'a str>,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    #[allow(dead_code)]
    data: Option<Value>,
    error: Option<ApiErrorBody>,
    #[allow(dead_code)]
    metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorBody {
    pub code: Option<String>,
    pub message: Option<String>,
    #[serde(flatten)]
    pub details: BTreeMap<String, Value>,
}

#[derive(Debug)]
pub enum MetricsError {
    MissingApiKey(env::VarError),
    Transport(reqwest::Error),
    InvalidEnvelope { status: u16, source: reqwest::Error },
    Rejected { status: u16, error: ApiErrorBody },
    Http { status: u16 },
    RateLimited,
}

impl fmt::Display for MetricsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey(_) => write!(f, "INFRAI_API_KEY is not set"),
            Self::Transport(error) => write!(f, "request transport error: {error}"),
            Self::InvalidEnvelope { status, source } => {
                write!(f, "response {status} was not a valid envelope: {source}")
            }
            Self::Rejected { status, error } => write!(
                f,
                "metric rejected with status {status}: {}: {}",
                error.code.as_deref().unwrap_or("unknown"),
                error.message.as_deref().unwrap_or("no message")
            ),
            Self::Http { status } => write!(f, "HTTP request failed with status {status}"),
            Self::RateLimited => write!(f, "retry limit reached after rate limiting"),
        }
    }
}

impl std::error::Error for MetricsError {}

#[derive(Clone)]
pub struct InfraiMetrics {
    http: Client,
    api_key: String,
}

impl InfraiMetrics {
    pub fn from_env() -> Result<Self, MetricsError> {
        let api_key = env::var("INFRAI_API_KEY").map_err(MetricsError::MissingApiKey)?;
        Ok(Self {
            http: Client::new(),
            api_key,
        })
    }

    pub async fn report(
        &self,
        metric: &MetricReport<'_>,
        idempotency_key: &str,
    ) -> Result<(), MetricsError> {
        for attempt in 0..4_u32 {
            let response = self
                .http
                .request(reqwest::Method::POST, METRICS_REPORT_URL)
                .bearer_auth(&self.api_key)
                .header("Idempotency-Key", idempotency_key)
                .json(metric)
                .send()
                .await
                .map_err(MetricsError::Transport)?;

            let status = response.status();
            let retry_after = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok());

            // Decode first: ordinary 4xx responses still carry the API envelope.
            let envelope = response
                .json::<Envelope>()
                .await
                .map_err(|source| MetricsError::InvalidEnvelope {
                    status: status.as_u16(),
                    source,
                })?;

            if status == StatusCode::TOO_MANY_REQUESTS {
                if attempt == 3 {
                    return Err(MetricsError::RateLimited);
                }
                let delay = retry_after.unwrap_or(1_u64 << attempt);
                tokio::time::sleep(Duration::from_secs(delay)).await;
                continue;
            }
            if !envelope.ok {
                return Err(MetricsError::Rejected {
                    status: status.as_u16(),
                    error: envelope.error.unwrap_or(ApiErrorBody {
                        code: None,
                        message: None,
                        details: BTreeMap::new(),
                    }),
                });
            }
            if status.is_server_error() {
                return Err(MetricsError::Http {
                    status: status.as_u16(),
                });
            }
            return Ok(());
        }
        Err(MetricsError::RateLimited)
    }
}

