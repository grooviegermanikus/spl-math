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

# Check the math-example program (excluded from the workspace, uses its own toolchain)
example-check:
    cd math-example && cargo clippy --all-targets --locked -- -D warnings

# Run the math-example on-chain tests against the pinned platform-tools (see math-example/README.md)
example-test-sbf:
    cd math-example && cargo test-sbf-pinned
