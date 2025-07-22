use crate::{
    abstract_trait::{
        DynAuthService, DynHashing, DynJwtService, DynSaldoRepository, DynSaldoService,
        DynTopupRepository, DynTopupService, DynTransferRepository, DynTransferService,
        DynUserRepository, DynUserService, DynWithdrawRepository, DynWithdrawService,
    },
    config::ConnectionPool,
    repository::{
        saldo::SaldoRepository, topup::TopupRepository, transfer::TransferRepository,
        user::UserRepository, withdraw::WithdrawRepository,
    },
    service::{
        auth::AuthService, saldo::SaldoService, topup::TopupService, transfer::TransferService,
        user::UserService, withdraw::WithdrawService,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct DependenciesInject {
    pub auth_service: DynAuthService,
    pub user_service: DynUserService,
    pub saldo_service: DynSaldoService,
    pub topup_service: DynTopupService,
    pub transfer_service: DynTransferService,
    pub withdraw_service: DynWithdrawService,
}

impl DependenciesInject {
    pub fn new(pool: ConnectionPool, hashing: DynHashing, jwt_config: DynJwtService) -> Self {
        let user_repository = Arc::new(UserRepository::new(pool.clone())) as DynUserRepository;

        let user_service =
            Arc::new(UserService::new(user_repository.clone(), hashing.clone())) as DynUserService;

        let auth_service = Arc::new(AuthService::new(
            user_repository.clone(),
            hashing.clone(),
            jwt_config,
        )) as DynAuthService;

        let saldo_repository = Arc::new(SaldoRepository::new(pool.clone())) as DynSaldoRepository;

        let topup_repository = Arc::new(TopupRepository::new(pool.clone())) as DynTopupRepository;

        let transfer_repository =
            Arc::new(TransferRepository::new(pool.clone())) as DynTransferRepository;

        let withdraw_repository =
            Arc::new(WithdrawRepository::new(pool.clone())) as DynWithdrawRepository;

        let saldo_service = Arc::new(SaldoService::new(
            user_repository.clone(),
            saldo_repository.clone(),
        )) as DynSaldoService;

        let topup_service = Arc::new(TopupService::new(
            topup_repository.clone(),
            saldo_repository.clone(),
            user_repository.clone(),
        )) as DynTopupService;

        let transfer_service = Arc::new(TransferService::new(
            transfer_repository.clone(),
            saldo_repository.clone(),
            user_repository.clone(),
        )) as DynTransferService;

        let withdraw_service = Arc::new(WithdrawService::new(
            withdraw_repository.clone(),
            saldo_repository.clone(),
            user_repository.clone(),
        )) as DynWithdrawService;

        Self {
            auth_service,
            user_service,
            saldo_service,
            topup_service,
            transfer_service,
            withdraw_service,
        }
    }
}
