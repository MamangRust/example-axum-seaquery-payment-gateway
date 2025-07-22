use anyhow::{Context, Result};
use dotenv::dotenv;

use example_sea_query_payment_gateway::config::{Config, ConnectionManager};
use example_sea_query_payment_gateway::handler::AppRouter;
use example_sea_query_payment_gateway::state::AppState;
use example_sea_query_payment_gateway::utils::tracing;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing();

    let config = Config::init().context("Failed to load configuration")?;

    let db_pool = ConnectionManager::new_pool(&config.database_url, config.run_migrations)
        .await
        .expect("Error initializing database connection pool");

    let port = config.port;

    let state = AppState::new(db_pool, &config.jwt_secret);

    println!("🚀 Server started successfully");

    AppRouter::serve(port, state)
        .await
        .context("Failed to start server")
}
