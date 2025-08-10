use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Serialize, Deserialize, Clone, Debug, IntoParams)]
pub struct FindAllSaldoRequest {
    #[serde(default = "default_page")]
    pub page: i32,

    #[serde(default = "default_page_size")]
    pub page_size: i32,

    #[serde(default)]
    pub search: String,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateSaldoRequest {
    #[serde(rename = "user_id")]
    #[validate(range(min = 1))]
    pub user_id: i32,

    #[serde(rename = "total_balance")]
    #[validate(range(min = 50000))]
    pub total_balance: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateSaldoRequest {
    #[serde(rename = "saldo_id")]
    #[validate(range(min = 1))]
    pub saldo_id: i32,

    #[serde(rename = "user_id")]
    #[validate(range(min = 1))]
    pub user_id: i32,

    #[serde(rename = "total_balance")]
    #[validate(range(min = 50000))]
    pub total_balance: i32,

    #[serde(rename = "withdraw_amount")]
    pub withdraw_amount: Option<i32>,

    #[serde(rename = "withdraw_time")]
    pub withdraw_time: Option<NaiveDateTime>,
}

impl UpdateSaldoRequest {
    pub fn extra_validate(&self) -> Result<(), String> {
        if let Some(amount) = self.withdraw_amount
            && amount < 50000
        {
            return Err("Withdraw amount must be at least 50000".to_string());
        }

        if self.withdraw_amount.is_none() && self.withdraw_time.is_none() {
            return Err("Either withdraw_amount or withdraw_time must be provided".to_string());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct UpdateSaldoBalance {
    #[validate(range(min = 50000))]
    pub total_balance: i32,

    #[validate(range(min = 1))]
    pub user_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct UpdateSaldoWithdraw {
    #[serde(rename = "user_id")]
    #[validate(range(min = 1))]
    pub user_id: i32,

    #[serde(rename = "total_balance")]
    #[validate(range(min = 50000))]
    pub total_balance: i32,

    #[serde(rename = "withdraw_amount")]
    pub withdraw_amount: Option<i32>,

    #[serde(rename = "withdraw_time")]
    pub withdraw_time: Option<DateTime<Utc>>,
}

impl UpdateSaldoWithdraw {
    pub fn extra_validate(&self) -> Result<(), String> {
        match self.withdraw_amount {
            Some(amount) if amount <= 0 => {
                return Err("Withdraw amount must be greater than 0".to_string());
            }
            Some(amount) if amount > self.total_balance => {
                return Err("Withdraw amount cannot be greater than total balance".to_string());
            }
            _ => {}
        }

        if self.withdraw_amount.is_some() && self.withdraw_time.is_none() {
            return Err(
                "Withdraw time must be provided if withdraw amount is provided".to_string(),
            );
        }

        if self.withdraw_amount.is_none() && self.withdraw_time.is_some() {
            return Err(
                "Withdraw amount must be provided if withdraw time is provided".to_string(),
            );
        }

        Ok(())
    }
}
