use crate::{
    abstract_trait::{DynHashing, DynJwtService},
    config::{ConnectionPool, Hashing, JwtConfig},
    utils::DependenciesInject,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub di_container: DependenciesInject,
    pub jwt_service: DynJwtService,
}

impl AppState {
    pub fn new(pool: ConnectionPool, jwt_secret: &str) -> Self {
        let jwt_service = Arc::new(JwtConfig::new(jwt_secret)) as DynJwtService;
        let hashing = Arc::new(Hashing::new()) as DynHashing;

        let di_container = DependenciesInject::new(pool, hashing, jwt_service.clone());

        Self {
            di_container,
            jwt_service,
        }
    }
}
