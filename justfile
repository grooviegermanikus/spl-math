set shell := ["bash", "-uc"]

default:
    @just --list

# Format
fmt:
    cargo fmt

# Format check (CI)
fmt-check:
    cargo fmt --all -- --check

# Lint
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Build
build:
    cargo build --all-targets

# Run tests
test:
    cargo test

# Run benchmarks
bench:
    cargo bench
