# Apex Scan #1 — High findings, refinement plans

Scope: the seven findings rated **High** in `Apex Report - spl-math / Scan #1`.
Critical / Medium / Low findings are out of scope for this set.

Every finding below was **independently reproduced** against the working tree
before the plan was written (throwaway harness under `tests/`, removed
afterwards). Observed values are quoted verbatim in each plan.

| ID | Title | Reproduced | Reported | Proposed | Workstream |
|----|-------|-----------|----------|----------|------------|
| [SPLM-4](SPLM-04-mul-div-ceil-overflow.md) | `mul_div_ceil` overflow fallback silently rounds down | yes | High | **High** | A |
| [SPLM-6](SPLM-06-newton-sqrt-subunit-bias.md) | Newton `sqrt()` solves `sqrt(x + ½ulp)` on sub-unit inputs | yes | High | **High** | B |
| [SPLM-8](SPLM-08-checked-mul-lossy-overflow.md) | `checked_mul` overflow rescue truncates, breaks `x·1 = x` | yes | High | Medium | C |
| [SPLM-19](SPLM-19-split-vs-fused-muldiv.md) | `checked_mul`+`checked_div` understates `mul_div_floor` | yes | High | Low (duplicate) | C |
| [SPLM-12](SPLM-12-new-from-f64-truncation.md) | `new_from_f64` truncates instead of rounding | yes | High | Medium | D |
| [SPLM-5](SPLM-05-d9-negative-f64.md) | D9 maps tiny negative `f64` to `0`; U256 presets reject | yes | High | Medium | D |
| [SPLM-14](SPLM-14-cordic-shift-ladder.md) | `sqrt_cordic` ladder cannot reach exact roots | yes | High | Medium | B |

## Workstreams

Fixing these one-by-one duplicates work. They collapse into four:

- **A — `mul_div_ceil`**: standalone, smallest, ship first.
- **B — sqrt accuracy** (SPLM-6, SPLM-14): both are sqrt precision defects that
  the existing differential tests structurally cannot catch, because each
  production algorithm is compared against an oracle that shares its defect.
  Fix the oracles and the proptest input strategy once, for both.
- **C — overflow rescue paths** (SPLM-8, SPLM-19, plus Medium SPLM-10 and
  SPLM-2): one root cause. `checked_mul` / `checked_div` degrade to lossy
  narrow-type arithmetic on overflow instead of using the double-width type
  that `define_muldiv!` already provides. SPLM-19 is the *symptom*, not a
  separate bug — it should be the acceptance test for the fix, not its own PR.
- **D — `f64` constructors** (SPLM-12, SPLM-5): both are about what
  `new_from_f64` promises. Decide the contract once, apply to all presets.

## Cross-cutting observation

Four of the seven survived the existing test suite for the same structural
reason: **the oracle is correlated with the implementation under test.**

- `newtonian_sqrt_approximation_generic` (the Newton oracle) calls
  `checked_div`, which carries the very `ROUNDING_CORRECTION` bias SPLM-6 is
  about — so the oracle agrees with the bug.
- `cordic_sqrt_approximation_naive` builds the same truncated `FP_ONE >> k`
  ladder as the fast version, so `test_cordic_optimized_vs_naive` (SPLM-14)
  cannot fail.
- `mul_div_ceil` has *no* proptest against `mul_div_ceil_naive` at all, though
  `mul_div_floor` does (SPLM-4).
- `test_square_root` samples the inner value uniformly from `0..u128::MAX`, so
  a sub-unit input (`value < 1e12`) has probability ≈ 1e-26 of ever being
  drawn (SPLM-6, SPLM-14).

Any fix here should also fix the oracle, otherwise the next regression is
equally invisible. See the "Test gap" section in each plan.
