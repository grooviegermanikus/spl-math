# SPLM-14 — `sqrt_cordic`'s shift ladder cannot reach exact roots

- Reported severity: **High** · Proposed severity: **Medium** (`sqrt_cordic` is
  not the default path; `sqrt()` dispatches to Newton)
- Workstream: **B** (with SPLM-6)
- Effort: **M** — algorithm change plus a perf question that must be benchmarked
- Blast radius: `sqrt_cordic` on every preset, plus its naive oracle

## 1. Reproduction (confirmed)

Exact d9 square `0.00390625 = (0.0625)²`, no intermediate rounding:

```
d9   radicand = 3906250     exact root = 62500000     cordic = 62499983   newton = 62500003
d12  radicand = 244140625   exact root = 15625000000  cordic = 15624999933 newton = 15625000015
```

Scanning several exact dyadic roots on d9 shows a consistent signature:

| root | exact | cordic | rel. err | newton | rel. err |
|---|---|---|---|---|---|
| 1/16 | 62500000 | 62499983 | 2.7e-7 | 62500003 | 4.8e-8 |
| 1/8 | 125000000 | 124999983 | 1.4e-7 | 125000001 | 8.0e-9 |
| 1/4 | 250000000 | 249999983 | 6.8e-8 | 250000000 | **0** |
| 1/2 | 500000000 | 499999983 | 3.4e-8 | 500000000 | **0** |

CORDIC is short by a near-constant **17 raw units** regardless of magnitude.
That constant-absolute-offset signature is the fingerprint of a lost residue in
the step basis, and it confirms the report's root-cause claim over the simpler
"it just stops early" explanation.

The d9 preset advertises 9 digits (`define_sqrt_tests!(..., (9, 9))`); at the
1/16 witness CORDIC delivers ~6.5. It misses its own published target.

## 2. Root cause

`src/precise_number/pn_impl.rs:299-346`. The candidate step is seeded from
`FP_ONE` and halved:

```rust
let mut pow2_inner = Self::FP_ONE;   // 1e9 / 1e12 — NOT a power of two
...
pow2_inner >>= 1;
```

`1e9 >> k` is exact only while `1e9` still has trailing zero bits
(`1e9 = 2^9 · 1953125`, so after 9 shifts). From `1953125 >> 1 = 976562` onward
every step is truncated. Binary search over sqrt only lands on the exact integer
root when the step basis is `{2^k}` and the descent reaches `2^0 = 1`; with a
truncated basis the reachable set has gaps and the residue is unrecoverable —
exactly the ~17-unit floor observed.

A **second, independent defect sits in the same loop** (`:330-339`): the early
exit fires when the accepted *step* is ≤ `PRECISION`, which is a step-size stop,
not an error stop, and it is only evaluated on iterations where the candidate is
accepted. Even an exhaustive replay with the break removed still cannot reach
the exact root (the report's own PoC shows `62499987`), which is what isolates
the basis as the primary cause. Fix both; do not assume fixing the break is
enough.

## 3. Fix

Replace the decimal-seeded ladder with an exact power-of-two binary search over
`floor(sqrt(x_shifted))`:

```rust
let x_shifted = x.value.checked_mul(Self::FP_ONE)?;
// highest power of two not exceeding the root
let mut step = <$FPInner>::one() << ((x_shifted.bits() + 1) / 2);
let mut result = Self::FP_ZERO;
while step != Self::FP_ZERO {
    let next = result + step;
    if next.checked_mul(next).map_or(false, |sq| sq <= x_shifted) {
        result = next;
    }
    step >>= 1;
}
```

This lands exactly on `floor(sqrt(x_shifted))` — the exact root for exact
squares — and drops the early-exit heuristic entirely.

**Cost:** the loop becomes `~bits/2` iterations (≈128 for U256, ≈64 for u128)
versus roughly 40 today. `sqrt_cordic` is documented as the ARM-preferred path
(`pn_impl.rs:388`), so this must be measured, not assumed:

- [ ] `just bench` / `cargo bench --bench bench_sqrt` before and after.
- [ ] If the regression is unacceptable, seed `step` from `x_shifted.bits()`
      rather than from `FP_ONE` (that alone restores exactness) and keep an
      error-based — not step-based — early exit.

`x_shifted.bits()` exists on the `uint`-generated types; a `NUM_BITS -
leading_zeros()` equivalent is needed for the primitive-backed presets (d4, d9).
Add it as a small macro-level helper.

## 4. Test gap

`test_cordic_optimized_vs_naive` (`pn_sqrt_tests.rs:282`) compares
`cordic_sqrt_approximation_fast` against `cordic_sqrt_approximation_naive`
(`test_oracles.rs:168`). The naive version builds **the same** ladder via
`div2()` (`pn_impl.rs:165`, a raw `>> 1` on the same non-power-of-two `FP_ONE`).
The oracle shares the defect, so the differential test can never fail on it.

`check_square_root` (`:171`) is the only accuracy check on CORDIC, and it uses
relative bounds at 11 digits with inputs drawn uniformly from `0..u128::MAX` —
the sub-unit and exact-dyadic cases where the residue floor dominates are never
sampled (see SPLM-6 §5 for the probability argument).

## 5. Test plan

- [ ] Rewrite `cordic_sqrt_approximation_naive` as a genuinely independent
      oracle — plain integer binary search on `x.value * FP_ONE`, no shared
      helpers with production. Without this the differential test is theatre.
      **Note this will make `test_cordic_optimized_vs_naive` fail until the
      production fix lands — that is the point.**
- [ ] Exact-square regression suite: for roots `2^-k · FP_ONE` (k = 1..8) build
      the radicand, assert `sqrt_cordic` returns the exact root, on d9/d12/d18.
- [ ] Add sub-unit and dyadic strategies to the sqrt proptests (shared with
      SPLM-6's test work — do it once).
- [ ] Assert the preset's own advertised digit target in
      `test_sqrt_precision_tuner` at these inputs, not only at
      `maximum_sqrt_base`.

## 6. Sequencing

Do SPLM-6 first. Both plans rewrite the sqrt test scaffolding (oracles + input
strategies); doing SPLM-14 second lets it reuse that work. The two production
fixes are in separate functions and do not conflict.

## 7. Definition of done

- [ ] Power-of-two step basis; step-size early exit removed or replaced with an
      error-based one.
- [ ] Naive CORDIC oracle rewritten to be independent, differential test green.
- [ ] Exact-dyadic-square regression suite green on all presets.
- [ ] `bench_sqrt` delta measured and accepted, or the cheaper seed-only variant
      adopted with the reason recorded.
