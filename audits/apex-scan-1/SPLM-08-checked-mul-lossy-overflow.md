# SPLM-8 — `checked_mul` overflow rescue truncates, breaking `x · 1 = x`

- Reported severity: **High** · Proposed severity: **Medium** (see §3 — the
  defect is real, the economic framing in the report is not supported)
- Workstream: **C** (with SPLM-19, and Medium findings SPLM-10 / SPLM-2)
- Effort: **M** — the clean fix requires a macro-signature change across 5 presets
- Blast radius: `checked_mul` on every preset; `checked_pow` transitively

## 1. Reproduction (confirmed)

```rust
let tiny = PreciseNumber::new_from_f64(3e-12).unwrap();          // value = 3
let b    = PreciseNumber::new(u128::MAX).unwrap()
             .checked_div(&tiny).unwrap()
             .checked_div(&tiny).unwrap();
b.checked_mul(&PreciseNumber::one())
```

Observed:

```
b      = 37809151880104273718152734159085356828333333333333388888888888833333333333
b · 1  = 37809151880104273718152734159085356828333333333333388888888888000000000000
lost   = 833333333333            (0.8333… of one fixed-point unit)
```

`Some(_)` is returned. The identity law is violated. Confirmed.

## 2. Root cause

`src/precise_number/pn_impl.rs:191-202`. On overflow of `a.value · b.value` the
code divides the **larger** operand by `FP_ONE` first:

```rust
self.value.checked_div(Self::FP_ONE)?.checked_mul(rhs.value)?
```

`larger.value / FP_ONE` = `floor(larger_real)`, discarding the fractional part
before it can participate in the product.

Exact error term: `lost = frac(larger_real) · smaller_real`, so
`0 ≤ lost < min(a_real, b_real)`.

## 3. Severity re-assessment — read before quoting the report

The report's assumption "*losing up to almost one whole fixed-point unit is
economically meaningful*" does not survive the bound above.

The fallback is only entered when `a.value · b.value ≥ 2^256`, which forces
`max(a_real, b_real) ≥ 2^128 / 1e12 ≈ 3.4e26`. Since the absolute loss is
bounded by the *smaller* operand and the result is at least
`larger · smaller`, the **relative** error is bounded by
`1/larger_real < 3e-27`.

Practical consequence: unreachable for realistic Solana token amounts
(`u64::MAX` ≈ 1.8e19 base units). To reach the fallback a caller must already
be carrying values ~7 orders of magnitude beyond `u64::MAX` in *whole units*.

What remains genuinely wrong, and worth fixing:

- **`x · 1 ≠ x`** — an algebraic identity silently failing is a correctness
  defect regardless of magnitude, and it is exactly the kind of thing downstream
  invariant checks and fuzzers assume.
- **Silent degradation.** The function returns `Some` from a code path with
  different semantics than its fast path. A caller cannot distinguish them.
- It is the proximate cause of SPLM-19.

Recommend re-rating **Medium** and rejecting the "economically meaningful"
assumption explicitly rather than leaving it checked.

## 4. Fix options

The correct arithmetic is `(a · b + C) / FP_ONE` evaluated in the double-width
type. The obstacle: `checked_mul` lives in `define_precise_number!`
(`pn_impl.rs:7`), which is **not** parameterised over the wide type — that lives
in `define_muldiv!` (`:466`).

- **(a) Thread the double-width type into `define_precise_number!`.**
  Add `$FPInnerDoublePrecision` to the macro signature, move
  `extend_precision` / `trunc_precision` into it, implement the overflow arms of
  both `checked_mul` and `checked_div` exactly. Call sites to update: 4 presets
  + `convert_from_f64.rs:98,111`, `pn_muldiv_tests.rs:8`,
  `pn_tests_pn_8_8_d1.rs`. Every one of them already has a natural wide type.
  **Recommended.** It fixes SPLM-8, SPLM-10, SPLM-19 and SPLM-2 in one change.

- **(b) Express `checked_mul` via a new `mul_div_round(b, one)` in
  `define_muldiv!`.** Less churn, but leaves `checked_mul` defined in a
  different macro from the rest of the type's arithmetic and creates a
  method-resolution split that will confuse the next reader.

- **(c) Return `None` on overflow.** One line, honest, no precision loss. But it
  is a behaviour break for any caller currently relying on `Some`, and it throws
  away results the wide type can represent exactly. Acceptable only as a
  stopgap.

Decide (a) vs (c) with the owner; they differ in effort by roughly a day, and
(c) is a semver-visible behaviour change while (a) is not.

## 5. Test gap

There is no oracle for `checked_mul` at all — `test_checked_mul`
(`pn_tests_pn_256_128_d12.rs:210`) is a handful of fixed small cases, none of
which reach the overflow arm.

## 6. Test plan

- [ ] Identity proptest across presets: `x.checked_mul(&one) == Some(x)` for all
      `x` including values above the overflow threshold. This is the assertion
      that fails today.
- [ ] Commutativity proptest: `a·b == b·a` (the current fallback branches on
      operand order, so this is worth pinning explicitly).
- [ ] Differential proptest `checked_mul` vs a wide-type oracle
      (`(a·b + C)/FP_ONE` computed in `$FPInnerDoublePrecision`), seeded to
      straddle the overflow boundary. Use `TestPreciseNumber8` so the boundary
      is trivially reachable.
- [ ] Regression test for the reported witness.

## 7. Definition of done

- [ ] Overflow arm is exact or explicitly `None` (decision recorded).
- [ ] Identity + commutativity + differential proptests green on all presets.
- [ ] SPLM-19's witness re-checked (see that plan — it is the acceptance test).
- [ ] `just bench` — the fast path must not regress; the wide path may.
