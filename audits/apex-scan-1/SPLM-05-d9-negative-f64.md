# SPLM-5 — D9 converts tiny negative `f64` to `0`; U256 presets reject

- Reported severity: **High** · Proposed severity: **Medium** (real preset
  divergence, but a much narrower input window than the report implies)
- Workstream: **D** (with SPLM-12)
- Effort: **XS** — 3 lines, one macro
- Blast radius: `PreciseNumber128D9` (public), `pn_64_32_d4` (private, not
  exported by `precise_number/mod.rs`)

## 1. Reproduction (confirmed, with a correction)

```
D9  new_from_inner_f64(-0.5)   => Some(0)     <-- accepted
D9  new_from_f64(-1e-10)       => Some(0)     <-- accepted
D9  new_from_f64(-0.5)         => None        <-- already rejected
D12 new_from_inner_f64(-0.5)   => None
D18 new_from_inner_f64(-0.5)   => None
```

The report's two witnesses reproduce. **But `new_from_f64(-0.5)` on D9 returns
`None`**, not `Some(0)` — the report does not claim otherwise, but the summary
("converts negative fractional f64 inputs to zero") reads broader than the
behaviour. Pin the real window before responding, or the finding invites a
partial-invalid ruling.

**Actual exposure:** the D9 path accepts a negative input only when the
*scaled* value lands in `(-1, 0)`, i.e. `x ∈ (-1e-9, 0)`. Anything more negative
is already rejected by `num_traits::ToPrimitive`, whose float→unsigned
conversion permits the exclusive range `(-1, MAX+1)`. For the private d4 preset
the window is `x ∈ (-1e-4, 0)`.

## 2. Root cause

`src/precise_number/pn_128_64_d9.rs:23` supplies `|value| value.to_u128()` as
the `f64` converter. `pn_64_32_d4.rs:22` supplies `|value| value.to_u64()`.
Neither carries the sign guard that `u256_from_f64_bits` has at
`src/precise_number/convert_from_f64.rs:8-10`:

```rust
if value.is_sign_negative() && !value.is_zero() {
    return None;
}
```

The guard exists per-converter rather than in the shared constructor
(`pn_impl.rs:406`), so each new preset has to remember it. Two out of five did
not.

## 3. Severity re-assessment

Arguments for keeping it below High:

- The accepted window is `(-1e-9, 0)`; the returned value `0` is the correctly
  rounded d9 representation of every number in it. The *arithmetic* is not
  wrong.
- No value is created from nothing; there is no sign confusion downstream,
  because the stored type is unsigned.

Argument for it being real and worth fixing:

- The report's assumption "*the integration treats rejection (`None`)
  differently from a legitimate zero amount*" is the correct one to defend.
  A caller using `new_from_f64(...).is_none()` as input validation gets a silent
  pass for negative input on D9 and a rejection on D12 — same code, different
  preset, opposite security outcome. That is a genuine trap.

Recommend **Medium**, fix immediately (cost is trivial), and keep the
validation-bypass framing rather than the "value corruption" framing.

## 4. Fix

Hoist the guard into the shared constructor so no preset can omit it, rather
than patching the two converters:

```rust
// pn_impl.rs, new_from_inner_f64
pub fn new_from_inner_f64(inner_value: f64) -> Option<Self> {
    if inner_value.is_sign_negative() && !inner_value.is_zero() {
        return None;
    }
    Self::CONVERT_FROM_F64(inner_value).map(|value| Self { value })
}
```

Two things to preserve:

- `-0.0` must still yield `Some(0)` — hence `&& !is_zero()`. There is an
  existing test for this (`convert_from_f64.rs:255`, `test_u256_from_negative_zero`).
- The guard inside `u256_from_f64_bits` becomes redundant for calls arriving via
  the constructor, but it is that function's own contract and it is directly
  unit-tested. Leave it in place.

Patching only `pn_128_64_d9.rs` and `pn_64_32_d4.rs` also works and is smaller,
but it preserves the structural defect: correctness of a security-relevant guard
stays a per-preset copy-paste obligation.

## 5. Test plan

- [ ] Add a macro-level test emitted by `define_precise_number!` (so it runs for
      **every** preset, including future ones):
      ```
      new_from_f64(-1e-10)      == None
      new_from_f64(-0.5)        == None
      new_from_f64(-1.0)        == None
      new_from_inner_f64(-0.5)  == None
      new_from_f64(-0.0)        == Some(zero)
      new_from_f64(f64::NAN)    == None
      ```
      This is the cheapest durable fix for the class, not just the instance.
- [ ] Cross-preset consistency test asserting D9/D12/D18 return the *same*
      `is_some()` verdict for an identical input.

## 6. Definition of done

- [ ] Sign guard hoisted to `new_from_inner_f64`.
- [ ] Per-preset negative-input test emitted from the macro, green everywhere.
- [ ] `-0.0` and NaN behaviour explicitly asserted.
- [ ] Rustdoc on both `f64` constructors states that negative non-zero input is
      rejected.
