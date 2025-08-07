use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::transfer::Transfer;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TransferResponse {
    pub transfer_id: i32,
    pub transfer_from: i32,
    pub transfer_to: i32,
    pub transfer_amount: i32,
    pub transfer_time: DateTime<Utc>,
    #[schema(format = "date-time")]
    pub created_at: Option<DateTime<Utc>>,

    #[schema(format = "date-time")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Transfer> for TransferResponse {
    fn from(value: Transfer) -> Self {
        TransferResponse {
            transfer_id: value.transfer_id,
            transfer_from: value.transfer_from,
            transfer_to: value.transfer_to,
            transfer_amount: value.transfer_amount,
            transfer_time: DateTime::from_naive_utc_and_offset(value.transfer_time, Utc),
            created_at: value
                .created_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            updated_at: value
                .updated_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }
    }
}
