use crate::{
    abstract_trait::{
        DynSaldoRepository, DynUserRepository, DynWithdrawRepository, WithdrawServiceTrait,
    },
    domain::{
        request::{
            CreateWithdrawRequest, FindAllWithdrawRequest, UpdateSaldoWithdraw,
            UpdateWithdrawRequest,
        },
        response::{
            ApiResponse, ApiResponsePagination, ErrorResponse, pagination::Pagination,
            withdraw::WithdrawResponse,
        },
    },
    utils::AppError,
};
use async_trait::async_trait;
use chrono::Utc;
use tracing::{error, info};

pub struct WithdrawService {
    withdraw_repository: DynWithdrawRepository,
    saldo_repository: DynSaldoRepository,
    user_repository: DynUserRepository,
}

impl WithdrawService {
    pub fn new(
        withdraw_repository: DynWithdrawRepository,
        saldo_repository: DynSaldoRepository,
        user_repository: DynUserRepository,
    ) -> Self {
        Self {
            withdraw_repository,
            saldo_repository,
            user_repository,
        }
    }
}

#[async_trait]
impl WithdrawServiceTrait for WithdrawService {
    async fn get_withdraws(
        &self,
        req: &FindAllWithdrawRequest,
    ) -> Result<ApiResponsePagination<Vec<WithdrawResponse>>, ErrorResponse> {
        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 { req.page_size } else { 10 };
        let search = if req.search.is_empty() {
            None
        } else {
            Some(req.search.clone())
        };

        let (withdraws, total_items) = self
            .withdraw_repository
            .find_all(page, page_size, search)
            .await?;

        info!("Found {} withdraws", withdraws.len());

        let total_pages = (total_items as f64 / page_size as f64).ceil() as i32;

        let withdraw_responses: Vec<WithdrawResponse> =
            withdraws.into_iter().map(WithdrawResponse::from).collect();

        Ok(ApiResponsePagination {
            status: "success".to_string(),
            message: "Withdraws retrieved successfully".to_string(),
            data: withdraw_responses,
            pagination: Pagination {
                page,
                page_size,
                total_items,
                total_pages,
            },
        })
    }

    async fn get_withdraw(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<WithdrawResponse>>, ErrorResponse> {
        let withdraw = self.withdraw_repository.find_by_id(id).await?;

        if let Some(withdraw) = withdraw {
            info!("Successfully retrieved withdraw with ID: {id}");
            Ok(ApiResponse {
                status: "success".to_string(),
                message: "Withdraw retrieved successfully".to_string(),
                data: Some(WithdrawResponse::from(withdraw)),
            })
        } else {
            error!("Withdraw with ID {id} not found");
            Err(ErrorResponse::from(AppError::NotFound(format!(
                "Saldo with id {id} not found",
            ))))
        }
    }

    async fn get_withdraw_users(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<Vec<WithdrawResponse>>>, ErrorResponse> {
        self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let withdraws = self.withdraw_repository.find_by_users(id).await?;

        let withdraw_response = if withdraws.is_empty() {
            None
        } else {
            Some(withdraws.into_iter().map(WithdrawResponse::from).collect())
        };

        let message = match &withdraw_response {
            Some(_) => "Success".to_string(),
            None => format!("No withdraw found for user with id {id}"),
        };

        Ok(ApiResponse {
            status: "success".to_string(),
            data: withdraw_response,
            message,
        })
    }

    async fn get_withdraw_user(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<WithdrawResponse>>, ErrorResponse> {
        let _user = self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let withdraw: Option<WithdrawResponse> = self
            .withdraw_repository
            .find_by_user(id)
            .await?
            .map(WithdrawResponse::from);

        match withdraw {
            Some(withdraw) => {
                info!("Successfully retrieved withdraw for user with id {id}");

                Ok(ApiResponse {
                    status: "success".to_string(),
                    data: Some(withdraw),
                    message: "Success".to_string(),
                })
            }
            None => {
                info!("No withdraw found for user with id {}", id);
                Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Topup with user id {id} not found",
                ))))
            }
        }
    }

    async fn create_withdraw(
        &self,
        input: &CreateWithdrawRequest,
    ) -> Result<ApiResponse<WithdrawResponse>, ErrorResponse> {
        info!("Creating withdraw for user_id: {}", input.user_id);

        let saldo = self
            .saldo_repository
            .find_by_user_id(input.user_id)
            .await
            .map_err(|_| {
                error!("Saldo with user_id {} not found", input.user_id);
                ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with user_id {} not found",
                    input.user_id
                )))
            })?;

        let saldo_ref = saldo.as_ref().ok_or_else(|| {
            error!("Saldo not found for user_id: {}", input.user_id);
            ErrorResponse::from(AppError::NotFound("Saldo not found".to_string()))
        })?;

        info!(
            "Saldo found for user_id: {}. Current balance: {}",
            input.user_id, saldo_ref.total_balance
        );

        if saldo_ref.total_balance < input.withdraw_amount {
            error!(
                "Insufficient balance for user_id: {}. Attempted withdrawal: {}",
                input.user_id, input.withdraw_amount
            );
            return Err(ErrorResponse::from(AppError::Custom(
                "Insufficient balance".to_string(),
            )));
        }
        info!("User has sufficient balance for withdrawal");

        let new_total_balance = saldo_ref.total_balance - input.withdraw_amount;

        let _update_saldo_balance = self
            .saldo_repository
            .update_saldo_withdraw(&UpdateSaldoWithdraw {
                user_id: input.user_id,
                withdraw_amount: Some(input.withdraw_amount),
                withdraw_time: Some(Utc::now()),
                total_balance: new_total_balance,
            })
            .await?;

        info!(
            "Saldo balance updated for user_id: {}. New balance: {new_total_balance}",
            input.user_id
        );

        let withdraw_create_result = self.withdraw_repository.create(input).await?;

        info!(
            "Withdraw created successfully for user_id: {}",
            input.user_id
        );

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "Withdraw created successfully".to_string(),
            data: withdraw_create_result.into(),
        })
    }

    async fn update_withdraw(
        &self,
        input: &UpdateWithdrawRequest,
    ) -> Result<ApiResponse<Option<WithdrawResponse>>, ErrorResponse> {
        let _withdraw = self
            .withdraw_repository
            .find_by_id(input.withdraw_id)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Withdraw with id {} not found",
                    input.withdraw_id
                )))
            })?;

        let saldo = self
            .saldo_repository
            .find_by_user_id(input.user_id)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with user_id {} not found",
                    input.user_id
                )))
            })?;

        let saldo_ref = saldo.as_ref().ok_or_else(|| {
            ErrorResponse::from(AppError::NotFound("Saldo not found".to_string()))
        })?;

        let new_total_balance = saldo_ref.total_balance - input.withdraw_amount;

        let updated_withdraw = self.withdraw_repository.update(input).await;

        if let Err(err) = updated_withdraw {
            let _rollback_saldo = self
                .saldo_repository
                .update_saldo_withdraw(&UpdateSaldoWithdraw {
                    user_id: input.user_id,
                    withdraw_amount: None,
                    withdraw_time: None,
                    total_balance: saldo_ref.total_balance,
                })
                .await?;

            error!("Rollback: Saldo reverted due to withdraw update failure");

            return Err(err.into());
        }

        let _update_saldo = self
            .saldo_repository
            .update_saldo_withdraw(&UpdateSaldoWithdraw {
                user_id: input.user_id,
                withdraw_amount: Some(input.withdraw_amount),
                withdraw_time: Some(Utc::now()),
                total_balance: new_total_balance,
            })
            .await?;

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "Withdraw updated successfully".to_string(),
            data: Some(updated_withdraw.unwrap().into()),
        })
    }

    async fn delete_withdraw(&self, id: i32) -> Result<ApiResponse<()>, ErrorResponse> {
        let user = self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let existing = self
            .withdraw_repository
            .find_by_user(user.unwrap().user_id)
            .await?;

        match existing {
            Some(_) => {
                self.withdraw_repository
                    .delete(existing.unwrap().withdraw_id)
                    .await?;

                info!("Withdraw deleted successfully for id: {id}");

                Ok(ApiResponse {
                    status: "success".to_string(),
                    message: "Withdraw deleted successfully".to_string(),
                    data: (),
                })
            }
            None => {
                error!("Withdraw with id {id} not found");
                Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Withdraw with id {id} not found",
                ))))
            }
        }
    }
}
