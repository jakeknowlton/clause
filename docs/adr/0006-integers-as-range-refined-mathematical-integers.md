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
- Hot loops often prove overflow-free and emit zero checks; the rest trap
  visibly.
- Wrapping is available but must be spelled explicitly.
