# Math

On-chain program wrapper around the [`spl-math`](..) crate, which lives in the same repo
and is depended on by path. The program exists for testing purposes: it makes the
library's compute-unit cost measurable on-chain.

This crate is **excluded** from the root workspace — its `solana-program` 3.x dependency
tree requires rustc 1.89, while `spl-math` pins 1.86 for agave compatibility. So it keeps
its own `Cargo.lock` and `rust-toolchain.toml`, and every command below must be run from
this directory.

## Develop

```bash
cargo test-sbf-pinned
```

```bash
cargo test-sbf
```

## Compute units depend on the platform-tools version

`tests/instruction_count.rs` asserts *exact* compute-unit counts. Those counts are a
property of the compiled `.so`, so they change when the SBF compiler changes — even
though the source does not. Upgrading the Solana CLI silently changes the compiler and
breaks every assertion in that file.

### Which version am I using?

```bash
cargo build-sbf --version     # prints the platform-tools version and its rustc
ls ~/.cache/solana/           # platform-tools versions downloaded locally
cat ~/.cache/solana/v1.52/platform-tools/version.md   # upstream rust/cargo/newlib commits
rustup toolchain list | grep sbpf                     # the linked +solana toolchain
```

`cargo build-sbf --version` reports the CLI's *default* platform-tools, which is what you
get when `--tools-version` is not passed:

```
solana-cargo-build-sbf 3.1.14
platform-tools v1.52
rustc 1.89.0
```

### Pinning a version

There is no environment variable for this — pass `--tools-version` explicitly. It works on
both `build-sbf` and `test-sbf`, and downloads the version on first use:

```bash
cargo test-sbf --tools-version v1.52
cargo build-sbf --install-only --tools-version v1.52   # download without building
```

To avoid relying on everyone remembering the flag, this repo aliases it in
`.cargo/config.toml` (a distinct name is required — aliasing `test-sbf` to itself recurses):

```toml
[alias]
test-sbf-pinned = ["test-sbf", "--tools-version", "v1.52"]
build-sbf-pinned = ["build-sbf", "--tools-version", "v1.52"]
```

Note `v1.43`–`v1.46` cannot build this project at all: their bundled rustc (1.79) is below
the MSRV of the current dependency tree. v1.47 is the oldest that works.

**Gotcha:** switching `--tools-version` does not invalidate cargo's dependency artifacts.
Mixing compilers fails with `E0460: found possibly newer version of crate compiler_builtins`
or `E0463: can't find crate`. Clearing only the SBF target directory is not enough — the
host-side build scripts and proc-macros under `target/release` are stale too (this shows up
as `indexmap` / `borsh` failing to build). Clear both when changing version:

```bash
rm -rf target/sbpf-solana-solana target/release
```

## Audit

See [audits/AUDIT_STATUS.md](../audits/AUDIT_STATUS.md) for the audit baseline of the
library this program wraps.
