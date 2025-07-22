use async_trait::async_trait;
use tracing::{error, info};

use crate::{
    abstract_trait::{
        DynSaldoRepository, DynTransferRepository, DynUserRepository, TransferServiceTrait,
    },
    domain::{
        request::{
            CreateTransferRequest, FindAllTransferRequest, UpdateSaldoBalance,
            UpdateTransferRequest,
        },
        response::{
            ApiResponse, ApiResponsePagination, ErrorResponse, pagination::Pagination,
            transfer::TransferResponse,
        },
    },
    utils::AppError,
};

pub struct TransferService {
    transfer_repository: DynTransferRepository,
    saldo_repository: DynSaldoRepository,
    user_repository: DynUserRepository,
}

impl TransferService {
    pub fn new(
        transfer_repository: DynTransferRepository,
        saldo_repository: DynSaldoRepository,
        user_repository: DynUserRepository,
    ) -> Self {
        Self {
            transfer_repository,
            saldo_repository,
            user_repository,
        }
    }
}

#[async_trait]
impl TransferServiceTrait for TransferService {
    async fn get_transfers(
        &self,
        req: &FindAllTransferRequest,
    ) -> Result<ApiResponsePagination<Vec<TransferResponse>>, ErrorResponse> {
        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 { req.page_size } else { 10 };
        let search = if req.search.is_empty() {
            None
        } else {
            Some(req.search.clone())
        };

        let (transfers, total_items) = self
            .transfer_repository
            .find_all(page, page_size, search)
            .await?;

        info!("Found {} transfers", transfers.len());

        let total_pages = (total_items as f64 / page_size as f64).ceil() as i32;

        let transfer_responses: Vec<TransferResponse> =
            transfers.into_iter().map(TransferResponse::from).collect();

        Ok(ApiResponsePagination {
            status: "success".to_string(),
            message: "Transfers retrieved successfully".to_string(),
            data: transfer_responses,
            pagination: Pagination {
                page,
                page_size,
                total_items,
                total_pages,
            },
        })
    }

    async fn get_transfer(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<TransferResponse>>, ErrorResponse> {
        let transfer = self.transfer_repository.find_by_id(id).await?;

        if let Some(transfer) = transfer {
            Ok(ApiResponse {
                status: "success".to_string(),
                message: "Transfer retrieved successfully".to_string(),
                data: Some(TransferResponse::from(transfer)),
            })
        } else {
            Err(ErrorResponse::from(AppError::NotFound(format!(
                "Transfer with id {id} not found",
            ))))
        }
    }

    async fn get_transfer_users(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<Vec<TransferResponse>>>, ErrorResponse> {
        let _user = self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let transfer = self.transfer_repository.find_by_users(id).await?;

        let transfer_response = if transfer.is_empty() {
            None
        } else {
            Some(transfer.into_iter().map(TransferResponse::from).collect())
        };

        let message = match &transfer_response {
            Some(_) => "Success".to_string(),
            None => format!("No transfer found for user with id {id}"),
        };

        Ok(ApiResponse {
            status: "success".to_string(),
            message,
            data: transfer_response,
        })
    }

    async fn get_transfer_user(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<TransferResponse>>, ErrorResponse> {
        let _user = self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let transfer: Option<TransferResponse> = self
            .transfer_repository
            .find_by_user(id)
            .await?
            .map(TransferResponse::from);

        let response = ApiResponse {
            status: "success".to_string(),
            data: transfer,
            message: "Success".to_string(),
        };

        Ok(response)
    }

    async fn create_transfer(
        &self,
        input: &CreateTransferRequest,
    ) -> Result<ApiResponse<TransferResponse>, ErrorResponse> {
        self.user_repository
            .find_by_id(input.transfer_from)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "User with id {} not found",
                    input.transfer_from
                )))
            })?;

        self.user_repository
            .find_by_id(input.transfer_to)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "User with id {} not found",
                    input.transfer_to
                )))
            })?;

        let transfer = self.transfer_repository.create(input).await?;

        let sender_saldo = self
            .saldo_repository
            .find_by_user_id(input.transfer_from)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with User id {} not found",
                    input.transfer_from
                )))
            })?;

        let sender_balance = sender_saldo.unwrap().total_balance - input.transfer_amount;

        let request_sender_balance = UpdateSaldoBalance {
            user_id: input.transfer_from,
            total_balance: sender_balance,
        };

        if let Err(db_err) = self
            .saldo_repository
            .update_balance(&request_sender_balance)
            .await
        {
            error!("Failed to update saldo balance for sender: {db_err}");
            self.transfer_repository
                .delete(transfer.transfer_id)
                .await?;

            return Err(db_err.into());
        }

        let receiver_saldo = self
            .saldo_repository
            .find_by_user_id(input.transfer_to)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with User id {} not found",
                    input.transfer_to
                )))
            })?;

        let receiver_balance = receiver_saldo.unwrap().total_balance + input.transfer_amount;

        let request_receiver_balance = UpdateSaldoBalance {
            user_id: input.transfer_to,
            total_balance: receiver_balance,
        };

        if let Err(db_err) = self
            .saldo_repository
            .update_balance(&request_receiver_balance)
            .await
        {
            error!("Failed to update saldo balance for receiver: {db_err}");
            let _ = self.transfer_repository.delete(transfer.transfer_id).await;

            return Err(db_err.into());
        }

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "Transfer created successfully".to_string(),
            data: TransferResponse::from(transfer),
        })
    }

    async fn update_transfer(
        &self,
        input: &UpdateTransferRequest,
    ) -> Result<ApiResponse<TransferResponse>, ErrorResponse> {
        let transfer = self
            .transfer_repository
            .find_by_id(input.transfer_id)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Transfer with id {} not found",
                    input.transfer_id
                )))
            })?
            .ok_or_else(|| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Transfer with id {} not found",
                    input.transfer_id
                )))
            })?;

        let amount_difference = input.transfer_amount as i64 - transfer.transfer_amount as i64;

        let sender_saldo = self
            .saldo_repository
            .find_by_user_id(transfer.transfer_from)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with User id {} not found",
                    transfer.transfer_from
                )))
            })?
            .ok_or_else(|| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with User id {} not found",
                    transfer.transfer_from
                )))
            })?;

        let new_sender_balance = sender_saldo.total_balance - amount_difference as i32;

        if new_sender_balance < 0 {
            return Err(ErrorResponse::from(AppError::Custom(
                "Insufficient balance for sender".to_string(),
            )));
        }

        let update_sender_balance = UpdateSaldoBalance {
            user_id: transfer.transfer_from,
            total_balance: new_sender_balance,
        };

        if let Err(db_err) = self
            .saldo_repository
            .update_balance(&update_sender_balance)
            .await
        {
            error!("Failed to update sender's saldo: {}", db_err);
            return Err(db_err.into());
        }

        let receiver_saldo = self
            .saldo_repository
            .find_by_user_id(transfer.transfer_to)
            .await
            .map_err(|_| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with User id {} not found",
                    transfer.transfer_to
                )))
            })?
            .ok_or_else(|| {
                ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with User id {} not found",
                    transfer.transfer_to
                )))
            })?;

        let new_receiver_balance = receiver_saldo.total_balance + amount_difference as i32;

        let update_receiver_balance = UpdateSaldoBalance {
            user_id: transfer.transfer_to,
            total_balance: new_receiver_balance,
        };

        if let Err(db_err) = self
            .saldo_repository
            .update_balance(&update_receiver_balance)
            .await
        {
            error!("Failed to update receiver's saldo: {db_err}");

            let rollback_sender_balance = UpdateSaldoBalance {
                user_id: transfer.transfer_from,
                total_balance: sender_saldo.total_balance,
            };

            self.saldo_repository
                .update_balance(&rollback_sender_balance)
                .await
                .map_err(|rollback_err| {
                    error!("Failed to rollback sender's saldo update: {rollback_err}");
                })
                .ok();

            return Err(db_err.into());
        }
        let updated_transfer = self.transfer_repository.update(input).await?;

        Ok(ApiResponse {
            status: "success".to_string(),
            message: "Transfer updated successfully".to_string(),
            data: TransferResponse::from(updated_transfer),
        })
    }

    async fn delete_transfer(&self, id: i32) -> Result<ApiResponse<()>, ErrorResponse> {
        let user = self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let existing_transfer = self
            .transfer_repository
            .find_by_user(user.unwrap().user_id)
            .await?;

        match existing_transfer {
            Some(_) => {
                self.transfer_repository
                    .delete(existing_transfer.unwrap().transfer_id)
                    .await?;

                info!("Topup deleted successfully for id: {id}",);

                Ok(ApiResponse {
                    status: "success".to_string(),
                    message: "Topup deleted successfully".to_string(),
                    data: (),
                })
            }
            None => {
                error!("Topup with id {id} not found");
                Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Topup with id {id} not found",
                ))))
            }
        }
    }
}
