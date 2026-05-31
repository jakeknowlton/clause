# Measures are a restricted total-by-construction sublanguage

**Status:** accepted

Which functions may appear inside a constraint predicate depends on the
constraint's **verification mode**: a **`static`** predicate may use only
`measure` functions (the solver reasons about them); a **`dynamic`** predicate
may use any `pure` function (it is only executed at runtime, never reasoned
about — a non-terminating dynamic check is a hang, not unsoundness). Predicates
are restricted to runtime-evaluable forms (bounded quantifiers only). A
`measure` is restricted to a sublanguage that is **total by construction**:
side-effect-free, structural recursion only (recursive calls must be on a
strictly smaller component of an input — no general loops or general recursion),
and total operations only (no trapping operations; partial ops such as indexing
or division require provable guards). The compiler enforces these by *form*; it
does **not** attempt to prove termination of arbitrary functions.

## Why

The solver reflects a recursive measure's defining equations as axioms. A
non-terminating (non-total) measure yields an inconsistent axiom set, from which
the solver can "prove" anything — i.e. it is *unsound*, not merely slow. Proving
termination of arbitrary functions is undecidable (the halting problem), so
instead of building a termination prover we restrict the measure language so
termination is guaranteed syntactically.

Note this concerns *measure* termination (soundness of the axioms). *Solver*
termination (decidability of a given query) is a separate matter, handled by the
ADR-0001 timeout ⇒ Unknown ⇒ runtime-check path.

## Consequences

- The `measure` restriction guarantees sound axioms for **static** predicates;
  **dynamic** predicates need only purity. Among *static* predicates, the
  proof/solver-timeout split is about solver decidability, not measure totality
  (every measure is total).
- Arbitrary iterative/general-recursive pure algorithms cannot be used directly
  in constraints; restructure into structural recursion, or use a `dynamic`
  predicate.
- Structural recursion covers the common measures (len, height, sum, contains,
  sorted, balanced, …), so the restriction retains practical usefulness.
- `pure` and `measure` are distinct keywords with distinct guarantees.
