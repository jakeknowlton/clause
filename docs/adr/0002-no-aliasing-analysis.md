# Constraints avoid aliasing analysis by construction

**Status:** accepted

Clause is garbage-collected with freely-aliased references, and deliberately
will **not** build an ownership/borrow/aliasing analysis. To keep refinement
constraints sound without one, we adopt this rule:

- A **mutable** binding may carry a constraint; the constraint is **re-checked
  at every assignment** (Refuted ⇒ compile error, Unknown ⇒ runtime check).
  Value-typed mutables have no aliasing, so this is cheap and sound.
- A **dependent** predicate (one that references values *other* than the one
  being constrained) may only reference **immutable** values/fields.

Because dependent proofs never rest on mutable state, no reachable mutation can
ever invalidate them — which removes the need for aliasing analysis entirely.

## Considered options

- **Constraints only on immutable bindings.** Too restrictive for a
  general-purpose language (no `let mut x where x > 0`).
- **Full mutable refinements + aliasing analysis.** Maximal power, but amounts
  to building a Rust-style ownership system — at odds with being GC'd and
  general-purpose.
- **Mutable constraints + immutable-only dependence (chosen).**

## Consequences

- Bounds checks against a *growable* mutable collection are not statically
  provable and become `dynamic` runtime checks — the same cost every other GC
  language pays.
- Encourages an immutable-by-default style (the prover-friendly form is the
  default), reinforced by the binding-keyword choice (see CONTEXT glossary).
