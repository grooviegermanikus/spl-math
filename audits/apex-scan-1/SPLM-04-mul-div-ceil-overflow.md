# SPLM-4 — `mul_div_ceil` overflow fallback silently rounds down

- Reported severity: **High** · Proposed severity: **High** (agree)
- Workstream: **A** (standalone)
- Effort: **S** — ~6 lines of code, ~40 lines of test
- Blast radius: all presets (`define_muldiv!` is instantiated by d4, d9, d12, d18)

## 1. Reproduction (confirmed)

```rust
let a     = PreciseNumber::new(u128::MAX).unwrap();
let b     = PreciseNumber::new(u128::MAX).unwrap();
let denom = PreciseNumber::new(u128::MAX - 1).unwrap();
```

Observed:

```
fast path overflows: true
remainder nonzero  : true
mul_div_floor      = 340282366920938463463374607431768211456000000000000
mul_div_ceil       = 340282366920938463463374607431768211456000000000000   <-- floor
true ceil (U512)   = 340282366920938463463374607431768211456000000000001
```

The witness uses only the public `new()` constructor. Confirmed.

## 2. Root cause

`src/precise_number/pn_impl.rs:497-517`. The `else` arm of `mul_div_ceil` is a
verbatim copy of `mul_div_floor`'s wide branch (`:489-493`) — the
`+ (denom - 1)` ceiling correction present on the fast path (`:505`) was never
carried into it.

There is a **second, narrower trigger in the same guard** that the report does
not mention: the fast path is

```rust
self.value.checked_mul(num.value).and_then(|x| x.checked_add(denom.value - 1))
```

so a product that fits in the inner type but whose *ceiling-corrected* value
does not also falls through to the floor branch. Same wrong answer, different
input class. Both are fixed by the same change; both need a test.

## 3. Fix

Mirror `mul_div_ceil_naive` (`src/precise_number/test_oracles.rs:231-240`) in
the wide branch:

```rust
let r = (Self::extend_precision(self.value) * Self::extend_precision(num.value)
    + (Self::extend_precision(denom.value) - <$FPInnerDoublePrecision>::from(1u8)))
    / Self::extend_precision(denom.value);
```

This cannot overflow the double-width type. For an `n`-bit inner type the worst
case is `(2^n-1)^2 + (2^n-2) = 2^2n - 2^n - 1 < 2^2n`. Holds for every
instantiation (u8/u16, u64/u128, u128/U256, U256/U512).

Rejected alternative: explicit `div`+`rem`+`is_zero` bump. Correct, but a second
wide division on a path that is already the slow path, and it diverges textually
from the oracle it is validated against. Prefer the form that makes the
production code and the oracle syntactically identical modulo the fast path.

Do **not** widen the fast-path guard to avoid the second trigger — falling into
a now-correct wide branch is the right behaviour.

## 4. Test gap and test plan

`src/precise_number/pn_muldiv_tests.rs:69-77` proptests `mul_div_floor` against
`mul_div_floor_naive`. **There is no equivalent for `mul_div_ceil`.** That is the
whole reason this shipped: `TestPreciseNumber8` has a `u8` inner type, so a
random `(a, b)` pair overflows the fast path almost always — a ceil-vs-oracle
proptest would have failed on roughly the first case.

1. Add to `pn_muldiv_tests.rs`, alongside `test_check_mul_div`:
   ```rust
   #[test]
   fn test_check_mul_div_ceil(a: u8, b: u8, c in 1..u8::MAX) { ... assert_eq!(prod, oracle) }
   ```
2. Extend `test_check_mul_div_invariants` (`:84`) to run its
   `ceil - floor ∈ {0, 1}` assertions against the **production** `mul_div_ceil`,
   not only `mul_div_ceil_naive`. Today it only validates the oracle against
   itself.
3. Add a fixed regression test for the reported d12 witness
   (`u128::MAX`, `u128::MAX`, `u128::MAX - 1`) asserting `ceil == floor + 1`.
4. Add a fixed regression test for the second trigger: a product that fits the
   inner type but where `product + denom - 1` does not.

## 5. Definition of done

- [ ] Wide branch carries the ceiling correction.
- [ ] Ceil-vs-oracle proptest added and passing on all presets.
- [ ] Invariant proptest exercises production `mul_div_ceil`.
- [ ] Both regression witnesses added.
- [ ] `just lint && just test` clean.
- [ ] `just bench` unchanged on the fast path (the edit touches only the
      overflow branch — a delta here would mean the fast path was disturbed).
