# CLAUDE.md

Guidance for Claude Code working in this repo.

## Project Overview

`spl-math` is a Rust library of high-precision math utilities for Solana
programs and Rust applications: fixed-point arithmetic (`PreciseNumber`),
overflow-checked operations, and mathematical approximations (e.g. `sqrt`).

## Repo Layout

- `src/` — the crate source.
- `benches/` — Criterion benchmarks (`bench_sqrt`, `benches_precision_number`).
- `audits/AUDIT_STATUS.md` — audit baseline (currently unaudited).
- `justfile` — top-level dev commands. Prefer these over raw `cargo`.

## Common commands

Always prefer `just` recipes. Run `just` with no args to list every recipe.

```bash
just fmt        # cargo fmt
just lint       # cargo clippy --all-targets --all-features -- -D warnings
just build      # cargo build --all-targets
just test       # cargo test
just bench      # cargo bench
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
