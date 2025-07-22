use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::topup::Topup;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TopupResponse {
    pub topup_id: i32,
    pub user_id: i32,
    pub topup_no: String,
    pub topup_amount: i32,
    pub topup_method: String,
    pub topup_time: DateTime<Utc>,
    #[schema(format = "date-time")]
    pub created_at: Option<DateTime<Utc>>,

    #[schema(format = "date-time")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Topup> for TopupResponse {
    fn from(value: Topup) -> Self {
        TopupResponse {
            topup_id: value.topup_id,
            user_id: value.user_id,
            topup_no: value.topup_no,
            topup_amount: value.topup_amount,
            topup_method: value.topup_method,
            topup_time: value.topup_time,
            created_at: value
                .created_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            updated_at: value
                .updated_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }
    }
}
