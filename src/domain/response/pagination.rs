use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Pagination {
    pub page: i32,
    pub page_size: i32,
    pub total_items: i64,
    pub total_pages: i32,
}
