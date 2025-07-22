run:
	cargo run --package example_sea_query_payment_gateway

build:
	cargo build --release --target x86_64-unknown-linux-musl --package example_sea_query_payment_gateway

up:
	docker compose up -d

down:
	docker compose down

clipy:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all -- --check