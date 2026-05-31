# No implementation inheritance; polymorphism via interfaces

**Status:** accepted

Clause has classes (reference types) and structs (value types), but **no
implementation inheritance**. All subtype polymorphism comes from `interface`s,
which structs and classes implement. Classes compose; they do not subclass.

## Why

Clause's invariants are machine-checked, not merely documented. Classical
inheritance forces the verifier into Liskov-substitution reasoning — subclasses
may only weaken preconditions and strengthen postconditions/invariants, virtual
calls must be reasoned about behaviorally, and "the invariant holds" becomes
subtle (fragile base class). Forbidding implementation inheritance keeps every
invariant local and unweakenable: a type's invariant is established and
maintained by that type alone, with nothing able to silently break it from a
parent or child.

## Considered options

- **Classical single inheritance (Java/C#/Eiffel).** Familiar OOP, but imports
  the full Liskov/variance burden into the verifier.
- **Interfaces-only, composition for reuse (chosen).**

## Consequences

- No code reuse by subclassing; recovered via composition and interface default
  methods.
- The verifier never reasons about invariant/contract variance across a class
  hierarchy.
- Surprising for an OOP-flavored language with `class` — hence this record.
