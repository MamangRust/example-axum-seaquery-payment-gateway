use crate::model::withdraw::Withdraw;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct WithdrawResponse {
    pub withdraw_id: i32,
    pub user_id: i32,
    pub withdraw_amount: i32,
    pub withdraw_time: DateTime<Utc>,
    #[schema(format = "date-time")]
    pub created_at: Option<DateTime<Utc>>,

    #[schema(format = "date-time")]
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<Withdraw> for WithdrawResponse {
    fn from(value: Withdraw) -> Self {
        WithdrawResponse {
            withdraw_id: value.withdraw_id,
            user_id: value.user_id,
            withdraw_amount: value.withdraw_amount,
            withdraw_time: value.withdraw_time,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
