.PHONY: build release test clean run-controller docker-build docker-up docker-down check verify

build:
	cargo build --workspace

release:
	cargo build --workspace --release

test:
	cargo test --workspace

clean:
	cargo clean

run-controller:
	cargo run --bin controller

check:
	cargo check --workspace

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all

verify: check clippy test
	@echo "✅ All checks passed!"

docker-build:
	docker-compose build

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f controller

mininet-clean:
	sudo mn -c

all: fmt clippy test build
