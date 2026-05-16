.PHONY: build release test clean run docker-build docker-up docker-down fmt clippy check

build:
	cargo build

release:
	cargo build --release

test:
	cargo test --workspace

clean:
	cargo clean

run:
	cargo run --bin dashboard_api

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

check:
	cargo check --workspace

docker-build:
	docker-compose build

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f

dev:
	RUST_LOG=debug cargo run --bin dashboard_api

watch:
	cargo watch -x 'run --bin dashboard_api'

bench:
	cargo bench --workspace

doc:
	cargo doc --workspace --no-deps --open

install-tools:
	cargo install cargo-watch
	cargo install cargo-audit

audit:
	cargo audit

all: fmt clippy test build
