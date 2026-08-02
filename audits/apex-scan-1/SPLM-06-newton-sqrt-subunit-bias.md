# SPLM-6 — Newton `sqrt()` solves `sqrt(x + ½ulp)` on sub-unit inputs

- Reported severity: **High** · Proposed severity: **High** (agree, and the
  report *understates* the affected range — see §3)
- Workstream: **B** (with SPLM-14)
- Effort: **S** for the fix, **M** for the test rework
- Blast radius: `sqrt()` (the default entry point) on every preset

## 1. Reproduction (confirmed)

```
d12  radicand.value = 1        exact root = 1000000       sqrt() = 1224744
d18  radicand.value = 1        exact root = 1000000000    sqrt() = 1224744873
d12  sqrt(0.25)               exact       = 500000000000  sqrt() = 500000000000  (exact)
```

`sqrt()` and `sqrt_newton()` agree; `1224744 / 1000000 = √1.5`, which identifies
the mechanism exactly.

## 2. Root cause

`src/precise_number/pn_impl.rs:271-274`:

```rust
let a_scaled = a.value.checked_mul(Self::FP_ONE)?
                      .checked_add(Self::ROUNDING_CORRECTION)?;   // <-- constant bias
```

`a_scaled` is hoisted out of the loop as a performance optimisation, but the
`ROUNDING_CORRECTION` it carries is *inside* the recurrence. Newton's fixed
point is `g = a_scaled / g`, so the loop converges to

```
g = sqrt(a.value · FP_ONE + FP_ONE/2)   instead of   sqrt(a.value · FP_ONE)
```

i.e. `sqrt(x + ½ ulp)`, not `sqrt(x)` rounded. The correction shifts the
*equation*, not the *result*.

## 3. True affected range — larger than reported

Relative error is `sqrt(1 + 1/(2·a.value)) − 1 ≈ 1/(4·a.value)`. Measured on d12:

| radicand `x` | radicand.value | rel. error |
|---|---|---|
| 1e-2  | 1e10 | 2.0e-11 |
| 1e-4  | 1e8  | 2.5e-9  |
| 1e-6  | 1e6  | 2.5e-7  |
| 1e-8  | 1e4  | 2.5e-5  |
| 1e-10 | 1e2  | 2.5e-3  |
| 1e-12 | 1    | **2.2e-1** |

The d12 preset advertises 11 digits (`define_sqrt_tests!(..., (12, 11))`).
That target is missed for every radicand below roughly **x = 0.025**, not just
for the minimum unit. This is the number to put in the finding: *sqrt is
out of spec across two orders of magnitude of ordinary sub-unit inputs*, which
is a materially stronger claim than the report's single-witness framing and
much harder to dismiss as dust.

Above `x = 1` the error is ≤ 2.5e-13 and vanishes — which is why nobody noticed.

## 4. Fix

```rust
let a_scaled = a.value.checked_mul(Self::FP_ONE)?;   // drop the bias
```

The recurrence is then the unbiased `x_{k+1} = (x_k + a_scaled/x_k)/2`. The
precompute hoist is preserved, so this is **perf-neutral** (one `checked_add`
removed from the setup, nothing added to the loop).

Open question to settle before implementing: the unbiased iteration converges to
`floor(sqrt(·))`-ish rather than nearest. Decide explicitly:

- **(a) accept floor semantics** — document `sqrt()` as truncating, consistent
  with `mul_div_floor`. Simplest; matches how the CORDIC path already behaves.
- **(b) round once at the end** — after the loop, compare `g²` and `(g+1)²`
  against `a_scaled` and pick the nearer. One extra multiply, outside the
  recurrence, so the fixed point is unaffected.

Recommend **(b)**: it keeps the current "sqrt is rounded" flavour that
`ROUNDING_CORRECTION` was clearly reaching for, without the bias. Confirm with
the owner before coding, since it changes results by 1 ulp for many inputs and
will need `proptest-regressions/` review.

## 5. Test gap

Two independent reasons this was invisible:

1. **The oracle shares the bug.**
   `test_newton_vs_cordic_vs_generic` (`pn_sqrt_tests.rs:266`) compares
   `newtonian_sqrt_approximation_fast` against
   `newtonian_sqrt_approximation_generic`. The generic version computes
   `self.checked_div(&num)` — and `checked_div` applies the *same*
   `ROUNDING_CORRECTION` (`pn_impl.rs:138`). Both drift the same way, so the
   differential test agrees with the defect. This is the single most important
   thing to fix; without it the regression returns silently.
2. **The input strategy never samples the affected range.**
   `test_square_root` and friends draw `a in 0..u128::MAX` as the *inner* value.
   `P(value < 1e12)` ≈ 1e-26. The sub-unit domain is effectively untested.
   `test_sqrt_precision_tuner` only probes `maximum_sqrt_base`,
   `maximum_sqrt_base/2` and `1.5` — all ≥ 1.

## 6. Test plan

- [ ] Add a BigDecimal-based (implementation-independent) sqrt oracle and use
      it for the Newton path, replacing the correlated generic-Newton oracle for
      accuracy assertions. Keep the generic version only as a shape/API check.
- [ ] Add a log-uniform proptest strategy covering `value in 1..FP_ONE`
      (sub-unit) as a first-class case, for every preset.
- [ ] Extend `test_sqrt_precision_tuner` with probes at `1e-2`, `1e-6`, `1e-12`
      and assert the preset's advertised digit target holds there — this is the
      test that should have failed.
- [ ] Fixed regression: exact sub-unit squares `(10^-k)² → 10^-k` for
      k = 1..6 on d12 and d18.
- [ ] `just bench` (`bench_sqrt`) to confirm perf-neutrality on SBF-relevant
      sizes.

## 7. Notes

- `sqrt_cordic` does **not** share this bias (it returned `999947` for the same
  input, off by 53 raw units, not 22%). Its own defect is SPLM-14. Do not
  "cross-fix" them — they are different bugs in different loops.
- `checked_div`'s use of `ROUNDING_CORRECTION` is the *Critical* SPLM-11
  finding. If SPLM-11 is fixed first, re-run this reproduction: the generic
  oracle will stop agreeing and several currently-green tests may turn red.
  Sequence B after the SPLM-11 decision.
