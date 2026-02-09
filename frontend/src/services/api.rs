use reqwasm::http::{Request, Response};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    NetworkError(String),
    Unauthorized,
    Forbidden,
    NotFound,
    BadRequest(String),
    ServerError(String),
    ParseError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::Unauthorized => write!(f, "Unauthorized - please log in"),
            ApiError::Forbidden => write!(f, "Access denied"),
            ApiError::NotFound => write!(f, "Resource not found"),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::ServerError(msg) => write!(f, "Server error: {}", msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Clone, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

pub struct ApiClient {
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token: None,
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub async fn get<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = Request::get(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::handle_response(response).await
    }

    pub async fn post<B, T>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let body_json =
            serde_json::to_string(body).map_err(|e| ApiError::ParseError(e.to_string()))?;

        let mut request = Request::post(&url)
            .header("Content-Type", "application/json")
            .body(body_json);

        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::handle_response(response).await
    }

    pub async fn post_empty<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut request = Request::post(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::handle_response(response).await
    }

    pub async fn put<B, T>(&self, endpoint: &str, body: &B) -> ApiResult<T>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let body_json =
            serde_json::to_string(body).map_err(|e| ApiError::ParseError(e.to_string()))?;

        let mut request = Request::put(&url)
            .header("Content-Type", "application/json")
            .body(body_json);

        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::handle_response(response).await
    }

    pub async fn delete<T>(&self, endpoint: &str) -> ApiResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut request = Request::delete(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::handle_response(response).await
    }

    pub async fn post_no_response<B>(&self, endpoint: &str, body: &B) -> ApiResult<()>
    where
        B: Serialize,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let body_json =
            serde_json::to_string(body).map_err(|e| ApiError::ParseError(e.to_string()))?;

        let mut request = Request::post(&url).header("Content-Type", "application/json");

        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .body(body_json)
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::handle_empty_response(response).await
    }

    pub async fn delete_no_response(&self, endpoint: &str) -> ApiResult<()> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut request = Request::delete(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::handle_empty_response(response).await
    }

    async fn handle_response<T>(response: Response) -> ApiResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();

        match status {
            200..=299 => {
                let text = response
                    .text()
                    .await
                    .map_err(|e| ApiError::ParseError(e.to_string()))?;

                serde_json::from_str(&text).map_err(|e| {
                    ApiError::ParseError(format!(
                        "Failed to parse response: {} - Body: {}",
                        e, text
                    ))
                })
            }
            401 => {
                // Clear auth from localStorage on 401
                clear_auth_storage();
                Err(ApiError::Unauthorized)
            }
            403 => Err(ApiError::Forbidden),
            404 => Err(ApiError::NotFound),
            400..=499 => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::BadRequest(text))
            }
            500..=599 => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(text))
            }
            _ => Err(ApiError::ServerError(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    async fn handle_empty_response(response: Response) -> ApiResult<()> {
        let status = response.status();

        match status {
            200..=299 => Ok(()),
            401 => {
                clear_auth_storage();
                Err(ApiError::Unauthorized)
            }
            403 => Err(ApiError::Forbidden),
            404 => Err(ApiError::NotFound),
            400..=499 => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::BadRequest(text))
            }
            500..=599 => {
                let text = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(text))
            }
            _ => Err(ApiError::ServerError(format!(
                "Unexpected status: {}",
                status
            ))),
        }
    }

    pub async fn post_multipart(
        &self,
        endpoint: &str,
        form_data: &web_sys::FormData,
    ) -> ApiResult<()> {
        let url = format!("{}{}", self.base_url, endpoint);

        let mut request = Request::post(&url);

        if let Some(token) = &self.token {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request
            .body(form_data.clone())
            .send()
            .await
            .map_err(|e| ApiError::NetworkError(e.to_string()))?;

        Self::handle_empty_response(response).await
    }
}

/// Clears authentication data from localStorage
/// Called automatically when a 401 Unauthorized response is received
fn clear_auth_storage() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("auth.token");
            let _ = storage.remove_item("auth.user");
        }
    }
}

// Default API client helper
pub fn api_client(token: Option<&str>) -> ApiClient {
    let base_url = get_api_base_url();
    let mut client = ApiClient::new(base_url);
    if let Some(t) = token {
        client = client.with_token(t);
    }
    client
}

fn get_api_base_url() -> String {
    // Use relative path - Trunk proxy will forward /api/* to backend
    "/api/v1".into()
}
