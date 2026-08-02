# SPL Math

High-precision math utilities for Solana programs and Rust applications. This library provides types and functions for precise arithmetic, including fixed-point numbers, checked division, and mathematical approximations.

## Features

- High-precision fixed-point arithmetic (`PreciseNumber` types)
- Safe math operations with overflow checks
- Mathematical approximations and utilities

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
spl-math = "version"
```

## Usage

Import and use in your Rust code:

```rust
use spl_math::precise_number::PreciseNumber;

fn use_it() {
    let a = PreciseNumber::new(10u128).expect("valid number");
    let _sqrt = a.sqrt();
}
```

## Development

### Fix Fmt+Clippy Issues

Run this all-in-one script to fix formatting and clippy issues:

```bash
./cargofixall
```


### Running Tests

Run all tests:

```bash
cargo test
```

### The `math-example` program

`math-example/` holds an on-chain Solana program that exercises this library, used to
measure compute-unit costs. It is **not** a workspace member: its `solana-program` 3.x
dependency tree requires rustc 1.89, while this crate pins 1.86 to stay buildable with
agave's toolchain. It therefore keeps its own `Cargo.lock` and `rust-toolchain.toml`, and
builds from its own directory:

```bash
cd math-example
cargo clippy --all-targets   # host-target check, no platform-tools needed
cargo test-sbf-pinned        # on-chain tests + compute-unit assertions
```

See [math-example/README.md](math-example/README.md) — the compute-unit assertions are
only reproducible against a pinned platform-tools version.

## Run benchmark

```bash
cargo bench
```

Use quick mode to run benchmark only until the significance level has been reached:
```bash
cargo bench --bench benches_precision_number -- --quick
```

## Acknowledgments

Thanks to [grooviegermanikus](https://github.com/grooviegermanikus) for the initial design and implementation of this library.

## License

This project is open-source under the MIT license.
