# Integers are range-refined mathematical integers; overflow is a constraint

**Status:** accepted

Machine integer types are defined as the unbounded mathematical integer `Int`
refined by a range:

    I32 ≡ { v: Int | -2^31 <= v < 2^31 }
    U8  ≡ { v: Int | 0 <= v < 256 }

Arithmetic (`+`, `-`, `*`, …) is **exact** in the logic: `a + b` has type
`{ v: Int | v == a + b }`. **Overflow is not a special case** — it is simply the
range refinement of the *target* type being re-imposed when an exact result is
stored back, discharged by the ordinary constraint trichotomy (Proven ⇒ no
check; Refuted ⇒ compile error; Unknown ⇒ runtime trap). A cast such as
`x as U8` likewise just imposes `0 <= x < 256`.

For intentional modular arithmetic, explicit **wrapping operators** (`+%`, `-%`,
`*%`) are defined as exact-arithmetic-mod-2ⁿ and carry no overflow obligation.

## Solver domain: LIA primary, bitvector bridge for bit-level ops

The solver's primary integer domain is **linear integer arithmetic over
mathematical `Int`** (LIA). This is what makes specifications *width-agnostic*
(`len(a) + len(b) == len(concat(a, b))` is one clean fact, no modulus) and keeps
overflow *separable* — "is the value right" (LIA) and "does it fit" (the range
refinement) stay distinct questions, which is the whole point above.

LIA cannot express the **bitwise/shift operators** (`&`, `|`, `^`, `~`, `<<`,
`>>`) or modular/**wrapping** arithmetic (`+%`, `-%`, `*%`); to a pure-LIA solver
these are uninterpreted and nothing about their results is provable. So for
*those operators specifically* the compiler bridges to the **bitvector (BV)**
theory — modelling the operation on the operand's fixed width and connecting it
to the `Int` world via `int2bv`/`bv2int`. Bit-twiddling code is then actually
reasoned about (e.g. `x & 0xFF < 256` is provable) instead of silently degrading
to a runtime check. Mixed LIA/BV queries the bridge cannot settle degrade to
Unknown ⇒ runtime check per the usual trichotomy, with the standard diagnostic.

(Dafny, F\*, and Lean all keep mathematical integers for specs and a separate
bitvector facility for bit-level reasoning; Liquid Haskell is LIA-only and is
correspondingly weak on bitwise code — the trap this bridge avoids.)

## `Int`/`Nat` are logical; runtime values are fixed-width

`Int` (and `Nat ≡ { v: Int | v >= 0 }`) are the solver's **reasoning** types:
they appear in predicates, measure signatures, and as the conceptual type of an
exact arithmetic result, but they have **no runtime representation** and may not
type a stored runtime value. Every runtime integer is a fixed-width machine type.
This keeps value types register-mapped and never refcounted, and keeps the C-FFI
ABI (ADR-0010) clean — a bignum maps to no C type. A program that genuinely needs
arbitrary precision uses an opt-in **`BigInt`** standard-library type: explicitly
named, heap-allocated, ARC-managed, and *distinct* from the solver's logical
`Int`.

## Scope: this model is integer-only

Everything above concerns **integers**. Floating-point (`F16`/`F32`/`F64`) is
**not** range-refined and sits **outside the static reasoning core** (v1): float
arithmetic is inexact (rounding, non-associative), and float `/`, overflow, and
domain errors are **IEEE-total** — they yield ±inf or NaN and **do not trap**
(contrast the *integer* `a / b` constraint in ADR-0008). NaN also makes ordering
non-total (`x > 0.0 || x <= 0.0` is invalid). Float-involving constraints are
therefore effectively `dynamic`/runtime in v1 — sound (a runtime `where x > 0.0`
rejects NaN), just not statically proven. Faithful IEEE reasoning via the SMT FP
theory (QF_FP) is a deferred frontier.

## Literal typing: context-polymorphic, default `I64`/`F64`

A numeric literal is **context-polymorphic**: it takes the numeric type its
context expects (annotation, parameter type, …), and its value is range-checked
against that type by the ordinary trichotomy — so `fix b: U8 = 300;` is a
**Refuted compile error** (300 ⊀ 256), not a silent truncation. A literal
adopting a type is *not* an implicit value conversion (ADR-0005 stays intact):
no value is being converted, so literals need no `as`.

When context pins nothing, the **unconstrained default is `I64` for integer
literals and `F64` for float literals** — `I64`, not the C/Rust `I32`, because
with exact-`Int` arithmetic defaulting too narrow turns ordinary constants into
compile errors (`1_000_000 * 1_000_000`, i.e. 10¹², is Refuted against `I32` but
fits `I64`). An unsuffixed literal can never default to `Int`/`Nat`: those are
logical-only, with no runtime representation.

## Integer operators across widths

The operators split by whether they yield a number:

- **Comparison / equality** (`< <= > >= == !=`) is allowed between **any** integer
  types — evaluated in `Int`, yielding `Bool`. No cast, and it is *mathematically
  correct*: `-1 < 4_000_000_000u32` is **true** in Clause, whereas C's implicit
  unsigned conversion makes it **false** (the classic signed/unsigned bug). Fixing
  that by construction is a direct payoff of the `Int` model, so a cast is never
  required here (and could never silently change the answer).
- **Arithmetic / bitwise** (`+ - * / % & | ^ << >>`) requires its operands to
  share a machine type. Context-polymorphic literals adapt to the other operand
  (so `x + 1` needs no annotation), so an explicit `as` is needed only between two
  *differently-typed non-literal values*. The result takes the operand type and
  carries the overflow obligation above.

The asymmetry is principled: comparison has no result type to pin and is always
safe in `Int`; arithmetic yields a number that needs a definite machine type, and
mixing widths is exactly where representation subtleties live — so it is made
explicit (ADR-0005). A unary `-`/`+` on a numeric literal folds into the signed
constant *before* range-checking (so `-128` checks against `I8` as `-128`, not as
a negated, already-out-of-range `128`).

## Why

Clause already has the solver; this model makes overflow fall out of the
constraint system uniformly rather than being a bolt-on. The solver proves
no-overflow where it can (zero runtime cost) and emits an honest visible trap
otherwise — silent wraparound, the classic bug class, is eliminated by default.
This is the clearest demonstration of the constraint-first thesis.

## Considered options

- **Wrapping-by-default (Go/Java):** total and simple, but every `+` only yields
  `v == (a+b) mod 2ⁿ`, muddying refinements, and silent wrap is exactly the bug
  constraints should kill.
- **Overflow is UB (C):** unacceptable for a safe, GC'd language.
- **Checked-by-default via range refinement (chosen).**

## Consequences

- The solver reasons over mathematical `Int`; the bounded types are refinements.
- Hot loops prove overflow-free for **index/affine arithmetic over the loop
  variable** (bounded by the `Range` fact `i in 0..n`) and emit zero checks;
  **loop-carried accumulators** (e.g. `sum += x`) are not statically bounded
  absent a loop invariant (deferred post-v1), so they keep a per-iteration
  runtime check — sound via the trichotomy, the cost every GC language pays
  (ADR-0002).
- Wrapping is available but must be spelled explicitly.
- Bitwise/shift/wrapping operators are discharged through a BV bridge, not LIA;
  where the bridge can't settle a mixed query it degrades to a runtime check.
- `Int`/`Nat` are compile-time-only; runtime arbitrary precision is the opt-in
  `BigInt` value type. Measure runtime-evaluation never uses bignum (ADR-0014).
- Cross-type integer comparison is legal and mathematically correct (no
  signed/unsigned surprise); cross-type arithmetic needs an explicit `as`.
