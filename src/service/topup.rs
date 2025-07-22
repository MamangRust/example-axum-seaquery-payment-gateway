use async_trait::async_trait;
use tracing::{error, info};

use crate::{
    abstract_trait::{
        DynSaldoRepository, DynTopupRepository, DynUserRepository, TopupServiceTrait,
    },
    domain::{
        request::{
            CreateSaldoRequest, CreateTopupRequest, FindAllTopupRequest, UpdateSaldoBalance,
            UpdateTopupAmount, UpdateTopupRequest,
        },
        response::{
            ApiResponse, ApiResponsePagination, ErrorResponse, pagination::Pagination,
            topup::TopupResponse,
        },
    },
    utils::AppError,
};

pub struct TopupService {
    topup_repository: DynTopupRepository,
    saldo_repository: DynSaldoRepository,
    user_repository: DynUserRepository,
}

impl TopupService {
    pub fn new(
        topup_repository: DynTopupRepository,
        saldo_repository: DynSaldoRepository,
        user_repository: DynUserRepository,
    ) -> Self {
        Self {
            topup_repository,
            saldo_repository,
            user_repository,
        }
    }
}

#[async_trait]
impl TopupServiceTrait for TopupService {
    async fn get_topups(
        &self,
        req: &FindAllTopupRequest,
    ) -> Result<ApiResponsePagination<Vec<TopupResponse>>, ErrorResponse> {
        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 { req.page_size } else { 10 };
        let search = if req.search.is_empty() {
            None
        } else {
            Some(req.search.clone())
        };

        let (topups, total_items) = self
            .topup_repository
            .find_all(page, page_size, search)
            .await?;

        info!("Found {} topups", topups.len());

        let total_pages = (total_items as f64 / page_size as f64).ceil() as i32;

        let topup_responses: Vec<TopupResponse> =
            topups.into_iter().map(TopupResponse::from).collect();

        Ok(ApiResponsePagination {
            status: "success".to_string(),
            message: "Topups retrieved successfully".to_string(),
            data: topup_responses,
            pagination: Pagination {
                page,
                page_size,
                total_items,
                total_pages,
            },
        })
    }

    async fn get_topup(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<TopupResponse>>, ErrorResponse> {
        info!("Fetching topup with id {id}");

        let topup = self.topup_repository.find_by_id(id).await;

        match topup {
            Ok(Some(topup)) => {
                info!("Successfully retrieved topup with id {id}");
                Ok(ApiResponse {
                    status: "success".to_string(),
                    message: "Topup retrieved successfully".to_string(),
                    data: Some(TopupResponse::from(topup)),
                })
            }
            Ok(None) => {
                error!("Topup with id {id} not found");
                Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Topup with id {id} not found",
                ))))
            }
            Err(err) => {
                error!("Error fetching topup with id {id}: {err}");
                Err(err.into())
            }
        }
    }

    async fn get_topup_users(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<Vec<TopupResponse>>>, ErrorResponse> {
        self.user_repository.find_by_id(id).await.map_err(|_| {
            error!("User with id {id} not found");
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let topups = self.topup_repository.find_by_users(id).await?;

        let topup_response = if topups.is_empty() {
            None
        } else {
            Some(topups.into_iter().map(TopupResponse::from).collect())
        };

        let message = match &topup_response {
            Some(_) => "Success".to_string(),
            None => "No topup found".to_string(),
        };

        Ok(ApiResponse {
            status: "success".to_string(),
            message,
            data: topup_response,
        })
    }

    async fn get_topup_user(
        &self,
        id: i32,
    ) -> Result<ApiResponse<Option<TopupResponse>>, ErrorResponse> {
        let _user = self.user_repository.find_by_id(id).await.map_err(|_| {
            error!("User with id {id} not found");
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let topup: Option<TopupResponse> = self
            .topup_repository
            .find_by_user(id)
            .await?
            .map(TopupResponse::from);

        match topup {
            Some(topup) => {
                info!("Successfully retrieved topup for user with id {id}");

                Ok(ApiResponse {
                    status: "success".to_string(),
                    data: Some(topup),
                    message: "Success".to_string(),
                })
            }
            None => {
                info!("No topup found for user with id {id}");
                Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Topup with user id {id} not found",
                ))))
            }
        }
    }

    async fn create_topup(
        &self,
        input: &CreateTopupRequest,
    ) -> Result<ApiResponse<TopupResponse>, ErrorResponse> {
        let _user = self
            .user_repository
            .find_by_id(input.user_id)
            .await
            .map_err(|_| {
                error!("User with id {} not found", input.user_id);
                ErrorResponse::from(AppError::NotFound(format!(
                    "User with id {} not found",
                    input.user_id
                )))
            })?;

        info!(
            "User with id {} found, proceeding with topup creation",
            input.user_id
        );

        let topup = self.topup_repository.create(input).await?;

        info!(
            "Topup created for user with id {}: topup amount {}",
            input.user_id, topup.topup_amount
        );

        match self.saldo_repository.find_by_user_id(input.user_id).await {
            Ok(Some(current_saldo)) => {
                let new_balance = current_saldo.total_balance + topup.topup_amount;
                let request = UpdateSaldoBalance {
                    user_id: input.user_id,
                    total_balance: new_balance,
                };

                if let Err(db_err) = self.saldo_repository.update_balance(&request).await {
                    error!(
                        "Failed to update saldo balance for user {}: {}",
                        input.user_id, db_err
                    );

                    if let Err(rb_err) = self.topup_repository.delete(topup.topup_id).await {
                        error!(
                            "Failed to rollback topup creation for user {}: {}",
                            input.user_id, rb_err
                        );
                    }

                    return Err(db_err.into());
                }

                info!(
                    "Saldo updated successfully for user {}. New balance: {new_balance}",
                    input.user_id,
                );
            }
            Ok(None) => {
                let create_saldo_request = CreateSaldoRequest {
                    user_id: input.user_id,
                    total_balance: topup.topup_amount,
                };

                if let Err(db_err) = self.saldo_repository.create(&create_saldo_request).await {
                    error!(
                        "Failed to create initial saldo for user {}: {}",
                        input.user_id, db_err
                    );

                    if let Err(rb_err) = self.topup_repository.delete(topup.topup_id).await {
                        error!(
                            "Failed to rollback topup creation for user {}: {rb_err}",
                            input.user_id
                        );
                    }

                    return Err(db_err.into());
                }

                info!(
                    "Initial saldo created for user {} with balance {}",
                    input.user_id, topup.topup_amount
                );
            }
            Err(_) => {
                error!("Failed to retrieve saldo for user {}", input.user_id);

                if let Err(rb_err) = self.topup_repository.delete(topup.topup_id).await {
                    error!(
                        "Failed to rollback topup creation for user {}: {}",
                        input.user_id, rb_err
                    );
                }

                return Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo with user_id {} not found",
                    input.user_id
                ))));
            }
        }

        info!(
            "Topup successfully created for user {}. Total balance updated.",
            input.user_id
        );
        Ok(ApiResponse {
            status: "success".to_string(),
            message: "Topup created successfully".to_string(),
            data: TopupResponse::from(topup),
        })
    }

    async fn update_topup(
        &self,
        input: &UpdateTopupRequest,
    ) -> Result<ApiResponse<Option<TopupResponse>>, ErrorResponse> {
        let _user = self
            .user_repository
            .find_by_id(input.user_id)
            .await
            .map_err(|_| {
                error!("User with id {} not found", input.user_id);
                ErrorResponse::from(AppError::NotFound(format!(
                    "User with id {} not found",
                    input.user_id
                )))
            })?;

        info!(
            "User with id {} found, proceeding with topup update",
            input.user_id
        );

        let existing_topup = self.topup_repository.find_by_id(input.topup_id).await?;

        let existing_topup = existing_topup.ok_or_else(|| {
            error!("Topup with id {} not found", input.topup_id);
            ErrorResponse::from(AppError::NotFound(format!(
                "Topup with id {} not found",
                input.topup_id
            )))
        })?;

        let topup_difference = input.topup_amount - existing_topup.topup_amount;

        info!(
            "Calculating topup difference: new amount {} - old amount {} = difference {topup_difference}",
            input.topup_amount, existing_topup.topup_amount,
        );

        let update_topup = UpdateTopupAmount {
            topup_id: input.topup_id,
            topup_amount: input.topup_amount,
        };

        self.topup_repository.update_amount(&update_topup).await?;

        match self.saldo_repository.find_by_user_id(input.user_id).await {
            Ok(Some(current_saldo)) => {
                let new_balance = current_saldo.total_balance + topup_difference;

                info!(
                    "Updating saldo: current balance {} + topup difference {topup_difference} = new balance {new_balance}",
                    current_saldo.total_balance,
                );

                let request = UpdateSaldoBalance {
                    user_id: input.user_id,
                    total_balance: new_balance,
                };

                if let Err(db_err) = self.saldo_repository.update_balance(&request).await {
                    error!(
                        "Failed to update saldo balance for user {}: {db_err}",
                        input.user_id,
                    );

                    let rollback = UpdateTopupAmount {
                        topup_id: existing_topup.topup_id,
                        topup_amount: existing_topup.topup_amount,
                    };

                    if let Err(rb_err) = self.topup_repository.update_amount(&rollback).await {
                        error!(
                            "Failed to rollback topup update for user {}: {}",
                            input.user_id, rb_err
                        );
                    }

                    return Err(db_err.into());
                }

                info!(
                    "Saldo updated successfully for user {}. New balance: {}",
                    input.user_id, new_balance
                );
            }
            Ok(None) => {
                error!("No saldo found for user {} to update", input.user_id);
                return Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Saldo for user {} not found",
                    input.user_id
                ))));
            }
            Err(e) => {
                error!("Failed to retrieve saldo for user {}: {}", input.user_id, e);
                return Err(e.into());
            }
        }

        let updated_topup = self.topup_repository.find_by_id(input.topup_id).await?;

        match updated_topup {
            Some(topup) => Ok(ApiResponse {
                status: "success".to_string(),
                message: "Topup updated successfully".to_string(),
                data: Some(TopupResponse::from(topup)),
            }),
            None => {
                error!("Topup with id {} not found", input.topup_id);
                Err(ErrorResponse::from(AppError::NotFound(format!(
                    "Topup with id {} not found",
                    input.topup_id
                ))))
            }
        }
    }

    async fn delete_topup(&self, id: i32) -> Result<ApiResponse<()>, ErrorResponse> {
        let user = self.user_repository.find_by_id(id).await.map_err(|_| {
            ErrorResponse::from(AppError::NotFound(format!("User with id {id} not found")))
        })?;

        let existing_topup = self
            .topup_repository
            .find_by_user(user.unwrap().user_id)
            .await?;

        match existing_topup {
            Some(_) => {
                self.topup_repository
                    .delete(existing_topup.unwrap().topup_id)
                    .await?;

                info!("Topup deleted successfully for id: {id}");

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
