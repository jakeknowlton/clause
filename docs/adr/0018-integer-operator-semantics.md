# Integer operator semantics: division, modulo, and shifts

**Status:** accepted

Refines ADR-0006 with the behaviour of the integer operators whose semantics are
*not* pinned by "exact arithmetic in `Int` + range refinement" alone.

## Division and modulo

- **`/` truncates toward zero** (C/Rust/Go, LLVM `sdiv`): `-7 / 2 == -3`.
- **`%` is the matching remainder**, sign following the **dividend**
  (`-7 % 2 == -1`, `7 % -2 == 1`), preserving `(a / b) * b + a % b == a`. It is a
  *remainder*, not a mathematical modulo; `core` provides `rem_euclid` (and a
  floor-div) for the always-non-negative version.
- **`b != 0`** is a constraint (ADR-0008): proven away or trapped.
- **`MIN / -1` overflows** — the result (e.g. `2³¹` for `I32`) is outside the
  target range and is caught by the ordinary overflow obligation (ADR-0006);
  traps if unproven. `MIN % -1 == 0` is fine.

**Solver requirement (soundness).** SMT-LIB's built-in `div`/`mod` are
**Euclidean** (remainder always ≥ 0), which does *not* match truncating semantics
for negative operands. The VC encoding for `/` and `%` must use the **truncating**
definition explicitly; reusing the Euclidean default would let the solver "prove"
false facts about signed division (e.g. `-7 / 2 == -4`). This is a concrete,
testable obligation on the VC encoder, not just a surface choice.

## Shifts

- **Shift amount in range is a constraint:** `0 <= b < bitwidth(a)` — C's
  out-of-range UB becomes a proof obligation (proven away for known small
  constants, trapped otherwise). The shift *amount* may be any integer type (it is
  a count, exempt from the arithmetic-same-type rule of ADR-0006).
- **`>>` is arithmetic for signed types** (sign-extending) and **logical for
  unsigned** (zero-fill), selected by the value's static type.
- **`<<` carries no overflow obligation:** it is a *bitwise* operator, so high
  bits drop (bit semantics, like the wrapping `+%` family). A checked
  multiply-by-2ᵏ is `x * 8`, not `x << 3`.

All shift/bitwise reasoning goes through the **bitvector (BV) bridge** (ADR-0006);
LIA cannot express it.

## Why

These match hardware and the C/Rust systems baseline (least surprise) while
folding the genuinely partial cases (`b != 0`, in-range shift amount, `MIN / -1`)
into the constraint trichotomy rather than leaving them UB. The split — arithmetic
(`+ - * /`) overflow/zero-checked, the bit family with bit semantics — keeps "did
the value stay correct" and "I asked for raw bits" distinct.

## Consequences

- `%` can be negative; reach for `rem_euclid` when a non-negative result is wanted.
- The solver layer must implement **truncating** div/mod, not SMT's Euclidean
  `div`/`mod`.
- Shifts and `MIN / -1` can trap; like all constraints they are proven away where
  the operands are statically known.
