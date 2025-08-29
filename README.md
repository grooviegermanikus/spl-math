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
spl-math-todo = "TODO_version"
```

## Usage

Import and use in your Rust code:

```rust
use spl_math_evolved::precise_number::PreciseNumber;

let a = PreciseNumber::new(10u128).expect("valid number");
let sqrt = a.sqrt();
```

## Development

### Running Tests

Run all tests:

```bash
cargo test
```

## Run benchmark

```bash
cargo bench
```

Use quick mode to run benchmark only until the significance level has been reached:
```bash
cargo bench --bench benches_precision_number -- --quick
```

## License

This project is open-source under the MIT license.
