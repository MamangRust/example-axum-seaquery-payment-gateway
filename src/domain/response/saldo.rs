use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::model::saldo::Saldo;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SaldoResponse {
    pub id: i32,
    pub user_id: i32,
    pub total_balance: i32,
    pub withdraw_amount: Option<i32>,
    pub withdraw_time: Option<DateTime<Utc>>,
    #[schema(format = "date-time")]
    pub created_at: Option<DateTime<Utc>>,

    #[schema(format = "date-time")]
    pub updated_at: Option<DateTime<Utc>>,p
}

impl From<Saldo> for SaldoResponse {
    fn from(value: Saldo) -> Self {
        SaldoResponse {
            id: value.saldo_id,
            user_id: value.user_id,
            total_balance: value.total_balance,
            withdraw_amount: value.withdraw_amount,
            withdraw_time: value
                .withdraw_time
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            created_at: value
                .created_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            updated_at: value
                .updated_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }
    }
}
