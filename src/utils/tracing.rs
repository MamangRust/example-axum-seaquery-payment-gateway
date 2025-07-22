use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

pub fn tracing() {
    // Layer untuk formatting output
    let fmt_layer = fmt::layer()
        .with_timer(fmt::time::uptime()) // Menampilkan waktu sejak start
        .with_line_number(true)
        .with_level(true)
        .with_target(true)
        .with_ansi(true) // Warna ANSI untuk terminal
        .compact(); // Format compact (bisa diganti dengan .pretty() untuk output lebih rinci)

    // Layer untuk filtering level
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info,my_crate=debug")) // Default ke info, tapi debug untuk crate Anda
        .unwrap();

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();

    // Log startup
    tracing::info!("Tracing initialized");
}
