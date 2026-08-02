# SPLM-19 — `checked_mul` then `checked_div` understates `mul_div_floor`

- Reported severity: **High** · Proposed severity: **Low / duplicate**
- Workstream: **C** — this is the *symptom* of SPLM-8 + SPLM-10, not a
  separate defect
- Effort: **0** as a fix; **S** as a permanent acceptance test
- Recommended disposition: **merge into SPLM-8**, keep the witness as the
  regression test that proves workstream C is done

## 1. Reproduction (confirmed)

```rust
let mut a = PreciseNumber::new(1e30 as u128).unwrap();
a.value += U256::from(999_999_999_999u128);
let (b, c) = (a, PreciseNumber::new(2).unwrap());
```

Observed:

```
split (checked_mul → checked_div) = 500000000000000000000000000000499999999999500000000000000000000000000000
fused (mul_div_floor)             = 500000000000000000000000000000999999999999000000000000000000499999999999
shortfall                         = 499999999999500000000000000000499999999999
```

Confirmed. Note the report's own PoC asserts the entire shortfall is introduced
by `checked_mul`, before `checked_div` is reached.

## 2. Why this is not an independent finding

There is no code path unique to this finding. It is the composition of two
already-reported defects:

- `checked_mul`'s overflow rescue truncates the larger operand — **SPLM-8**
  (High), and in this witness it accounts for 100% of the observed shortfall.
- `checked_div`'s overflow rescue divides before scaling, quantising the
  quotient to whole units — **SPLM-10** (Medium).

Both live in `src/precise_number/pn_impl.rs:142-149` and `:191-202`. Fixing
either changes this witness; fixing both makes it exact. Filing it separately
would triple-count one root cause in the severity tally.

## 3. Severity re-assessment

The absolute shortfall is large in raw units (4.99e41) but the result is ~5e71
raw, so the **relative** error is ~1e-30 — the same bound derived in the SPLM-8
plan. The report's framing ("*materially wrong accounting*") is not supported by
this witness; the magnitude is an artifact of quoting a raw fixed-point integer
rather than a relative error.

A witness that *would* justify High severity is the SPLM-10 one, where
`checked_div`'s rescue turns `1.6` into `1.0` — a 37% relative error. If the
report is being re-issued, that witness belongs here and this one does not.

## 4. Action

Do **not** open a separate fix. Instead:

- [ ] Fold into the SPLM-8 / SPLM-10 workstream (C).
- [ ] Add a permanent cross-helper consistency proptest, which is the durable
      value of this finding:
      ```
      for all a, b, c (c != 0):
          |a.checked_mul(&b)?.checked_div(&c)? − a.mul_div_floor(b, c)?| ≤ 1 ulp
      ```
      Seed it with inputs that straddle both overflow thresholds. Use
      `TestPreciseNumber8` (u8 inner) so both rescue paths are hit constantly.
      This assertion fails today and must pass when C lands — it is the
      acceptance criterion for the whole workstream.
- [ ] Record in the report response that SPLM-19 is accepted as valid but
      merged, with the reasoning above. Do not mark it invalid: the behaviour is
      real and the consistency test it motivates is worth keeping.

## 5. Definition of done

- [ ] Cross-helper consistency proptest added and green after workstream C.
- [ ] Reported witness added as a fixed regression test.
- [ ] Finding closed as duplicate-of-SPLM-8 with written justification.
