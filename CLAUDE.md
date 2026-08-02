# CLAUDE.md

Guidance for Claude Code working in this repo.

## Project Overview

`spl-math` is a Rust library of high-precision math utilities for Solana
programs and Rust applications: fixed-point arithmetic (`PreciseNumber`),
overflow-checked operations, and mathematical approximations (e.g. `sqrt`).

## Repo Layout

- `src/` — the crate source.
- `benches/` — Criterion benchmarks (`bench_sqrt`, `benches_precision_number`).
- `math-example/` — an on-chain Solana program that exercises the library and asserts
  compute-unit costs. Deliberately **excluded** from the workspace (see below).
- `audits/AUDIT_STATUS.md` — audit baseline (currently unaudited).
- `justfile` — top-level dev commands. Prefer these over raw `cargo`.

## Two toolchains

The repo has two independent build worlds, selected by directory:

| | root (`spl-math`) | `math-example/` |
|---|---|---|
| toolchain | 1.86.0 (agave v2.3.13 compat) | 1.89.0 (required by `solana-program` 3.x) |
| lockfile | `Cargo.lock` | `math-example/Cargo.lock` |

`math-example` is listed under `[workspace] exclude` in the root `Cargo.toml` because the
two toolchains are incompatible — its dep tree will not resolve under 1.86. Never add it to
`members`, and never run a `--workspace` command expecting to cover it. Build it from its
own directory, where its nested `rust-toolchain.toml` and `.cargo/config.toml` apply.

Note `math-example` is not currently `cargo fmt` clean, and the root `fmt` job does not
reach it. CI checks it with clippy only.

## Common commands

Always prefer `just` recipes. Run `just` with no args to list every recipe.

```bash
just fmt        # cargo fmt
just lint       # cargo clippy --all-targets --all-features -- -D warnings
just build      # cargo build --all-targets
just test       # cargo test
just bench      # cargo bench

just example-check      # clippy the math-example program (its own toolchain)
just example-test-sbf   # math-example on-chain tests, pinned platform-tools
```

Benchmark quick mode (stops once the significance level is reached):

```bash
cargo bench --bench benches_precision_number -- --quick
```

## Branch Workflow

- `main` — integration branch.
- `feat/*`, `fix/*`, `chore/*` — topic branches from `main`.
- `hotfix/*` — urgent fixes from a deployed stable tag.

PRs target `main` (enforced by `.github/workflows/pr-target-check.yml`).
