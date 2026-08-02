# SPLM-12 — `new_from_f64` maps exact-looking decimals to adjacent values

- Reported severity: **High** · Proposed severity: **Medium** (one actionable
  defect, one unfixable-by-design limitation — the report conflates them)
- Workstream: **D** (with SPLM-5)
- Effort: **S** for the fix, **S** for the doc work; the decision costs more
  than the code
- Blast radius: `new_from_f64` on every preset; changes existing asserted behaviour

## 1. Reproduction (confirmed)

d12 preset:

```
2.01                 => 2009999999999      (exact would be 2010000000000, −1 ulp)
9007.199254740993    => 9007199254740994   (exact would be …993,          +1 ulp)
0.1                  => 100000000000       (exact)
1.1                  => 1100000000000      (exact)
```

Confirmed.

## 2. Two distinct causes — separate them before responding

**(A) Truncation instead of round-to-nearest — actionable.**
`2.01f64` is `2.00999999999999978…`. Times `1e12` that is
`2009999999999.99978`. `u256_from_f64_bits`
(`src/precise_number/convert_from_f64.rs:5`) is documented as *"Converts from
the integer part of f64"* and truncates, yielding `…999`. A single
round-to-nearest would yield the value the caller meant. This affects every
decimal whose scaled form is not exactly representable — i.e. almost all of
them — with an error of at most 1 ulp.

**(B) `f64` mantissa exhaustion — not fixable in a constructor.**
Above `2^53 / FP_ONE` (≈ 9007.19 for d12, ≈ 9.0 for d18) the `f64` argument
simply does not carry the requested digits. The report's third witness
(`1e18 + 1e-12` → error 1.99e13 raw) is entirely this: the `f64` literal *is*
`1e18` before the crate sees it. No implementation change helps. This is a
documentation and API-shape issue, not a bug.

Responding to (B) as if it were a defect will (correctly) get the finding
disputed — the report even predicts this in its own "Predicted Invalid Reasons".
Respond to (A) on its merits and concede (B) as documented behaviour.

## 3. Fix options for (A)

- **(a) Round in `new_from_f64` only** (`pn_impl.rs:401-404`):
  ```rust
  let scaled_value = (input_f64 * Self::FP_ONE_F64).round();
  ```
  Minimal blast radius: `new_from_inner_f64` and `u256_from_f64_bits` keep their
  documented truncating "integer part of f64" semantics, so
  `test_truncated_prop` (`convert_from_f64.rs:324`) and the raw-bits tests are
  untouched. **Recommended.**

- **(b) Round inside `u256_from_f64_bits`.** Wrong layer — that function is a
  bit-level `f64 → U256` reinterpretation and its truncation is deliberate and
  tested. Rejecting.

- **(c) Reject inexact inputs.** Rejecting. Practically every decimal is inexact
  in binary; this would make `new_from_f64(0.1)` return `None`.

- **(d) Add a decimal-string / integer-scaled constructor** and steer
  value-bearing call sites to it. This is the real answer for (B) and is worth
  doing as a follow-up, not as a blocker.

## 4. Behaviour changes that (a) causes — decide explicitly

Rounding is not a free win; it flips existing *asserted* behaviour:

| existing assertion | file | today | after (a) |
|---|---|---|---|
| `new_from_f64(0.07) == 0` (d1 test type) | `convert_from_f64.rs:132` | `0` | **`1`** (`0.07·10 = 0.70000000000000007` → rounds up) |
| `new_from_f64(25.6).is_none()` | `:137` | `None` | `None` (unchanged, `256 > u8::MAX`) |
| `new_from_f64(12.3) == 123` | `:126` | `123` | `123` |

The `0.07` case is arguably *more* correct after the change — `0.07` is nearer
to `0.1` than to `0.0` in a 1-decimal type — but it is a deliberate semantic
change to a tested behaviour and needs the owner's sign-off, not a silent test
edit.

## 5. Test plan

- [ ] Round-trip proptest: for decimals with ≤ `DECIMALS` fractional digits whose
      scaled value is `< 2^53`, `new_from_f64(d)` must equal the exact scaled
      integer. This is the property that pins (A) shut.
- [ ] Boundary tests at `2^53 / FP_ONE` per preset, asserting documented
      behaviour above it rather than exactness.
- [ ] Update `test_pn_from_f64` with the new expected `0.07` result and a
      comment explaining the semantics change.
- [ ] Confirm `new_from_inner_f64` / `u256_from_f64_bits` behaviour and their
      proptests are **unchanged** — that is the guard rail proving the blast
      radius stayed contained.

## 6. Documentation (required, not optional)

Rustdoc on `new_from_f64` must state: the exactness envelope
(`|x| < 2^53 / FP_ONE`), the rounding mode, and that outside the envelope the
`f64` argument itself has already lost the digits. Point callers at the
integer/string constructor from (d) for value-bearing paths.

## 7. Definition of done

- [ ] Rounding applied at the `new_from_f64` layer only.
- [ ] Round-trip proptest green on all presets.
- [ ] `0.07` semantics change signed off and documented in the changelog.
- [ ] Exactness envelope documented per preset.
- [ ] Follow-up ticket opened for a decimal-string constructor.
