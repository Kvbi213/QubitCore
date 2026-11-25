# Makefile for QubitCore
# Enforces CARGO_INCREMENTAL=0 to protect host RAM (3.5 GB AiO limit)

export CARGO_INCREMENTAL := 0
export RUST_BACKTRACE := 1

.PHONY: all check test audit fmt lint clean

all: check test

check:
	cargo check --workspace --all-targets

test:
	cargo nextest run --workspace || cargo test --workspace

audit:
	cargo audit
	cargo deny check

fmt:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

machete:
	cargo machete

geiger:
	cargo geiger

clean:
	cargo clean
