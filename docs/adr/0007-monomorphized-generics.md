# Monomorphized generics; interface bounds; boxed interface-values

**Status:** accepted

Generics are compiled by **monomorphization** — the compiler emits a specialized
copy per concrete instantiation. Type parameters are bounded by **interfaces**
(`fun sort<T: Ord>(...)`), the only polymorphism source (ADR-0004). When dynamic
dispatch is wanted, using an `interface` *as a value* produces a **boxed,
vtable-dispatched** existential (the deliberate opt-in to indirection, parallel
to Rust's `dyn Trait`).

Refinements over generics, v1 scope: **container/structure-level** refinements
work fully (e.g. `len(xs)`, `i < len(xs)`), because each monomorphized
instantiation has concrete types. **Value-level refinement over an abstract `T`**
(e.g. `max<T: Ord>(a,b) -> T where it >= a`) requires the bounding interface to
publish measure-level specs and is deferred to a later extension.

## Why

Monomorphization keeps value types unboxed (Clause leans on value semantics) and
— uniquely important here — gives the solver fully concrete types per
instantiation, so refinements are visible and provable. Erasure/uniform
representation would box everything (fighting value semantics) and hide
per-instantiation refinements.

## Consequences

- Code bloat and longer compile times; pure separate compilation is harder
  (acceptable for a verified systems language).
- Dynamic dispatch is possible but explicit (boxed interface-values).
- The frontier of refining over abstract type parameters is acknowledged and
  intentionally limited in v1.
