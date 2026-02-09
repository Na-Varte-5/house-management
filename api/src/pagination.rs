use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Default number of items per page.
const DEFAULT_PER_PAGE: i64 = 20;
/// Maximum allowed items per page.
const MAX_PER_PAGE: i64 = 100;

/// Query parameters for paginated list endpoints.
///
/// - `page`: 1-based page number (default: 1)
/// - `per_page`: items per page (default: 20, max: 100)
#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationParams {
    /// Page number (1-based, default: 1)
    pub page: Option<i64>,
    /// Items per page (default: 20, max: 100)
    pub per_page: Option<i64>,
}

impl PaginationParams {
    /// Returns the validated page number (minimum 1).
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    /// Returns the validated per_page value, clamped to [1, MAX_PER_PAGE].
    pub fn per_page(&self) -> i64 {
        self.per_page
            .unwrap_or(DEFAULT_PER_PAGE)
            .clamp(1, MAX_PER_PAGE)
    }

    /// Returns the SQL OFFSET value for the current page.
    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }

    /// Returns the SQL LIMIT value.
    pub fn limit(&self) -> i64 {
        self.per_page()
    }
}

/// Wrapper for paginated API responses.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize + ToSchema> {
    /// The items for the current page.
    pub data: Vec<T>,
    /// Pagination metadata.
    pub pagination: PaginationMeta,
}

/// Metadata about the current page of results.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    /// Current page number (1-based).
    pub page: i64,
    /// Items per page.
    pub per_page: i64,
    /// Total number of items across all pages.
    pub total: i64,
    /// Total number of pages.
    pub total_pages: i64,
}

impl<T: Serialize + ToSchema> PaginatedResponse<T> {
    /// Build a paginated response from a full item list and pagination params.
    pub fn new(data: Vec<T>, total: i64, params: &PaginationParams) -> Self {
        let per_page = params.per_page();
        let total_pages = if total == 0 {
            1
        } else {
            (total + per_page - 1) / per_page
        };
        Self {
            data,
            pagination: PaginationMeta {
                page: params.page(),
                per_page,
                total,
                total_pages,
            },
        }
    }
}
