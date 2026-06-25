# Audit Status

## Current Baseline

**spl-math has not yet been audited.**

No third-party security audit has been completed for this crate. There is no
audited-through commit to track yet.

## Branch and Release Model

- `main` is the integration branch.
- Stable production releases are immutable tags/releases (for example `v1.0.0`).
- Audited baselines will be tracked by commit SHA plus immutable tags/releases,
  not by long-lived release branches.

## Maintenance Rules

When the first audit is completed:

1. Add the audit report to `audits/`.
2. Replace the "not yet audited" note above with:
   - `Auditor:` name
   - `Report:` path to the report file
   - `Audited-through commit:` the audited SHA
   - `Compare unaudited delta:` a GitHub compare link from that SHA to `main`
3. Tag the audited release commit(s) (for example `vX.Y.Z`).
4. Update README and release notes links if needed.
