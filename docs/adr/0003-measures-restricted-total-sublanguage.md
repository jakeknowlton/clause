# Measures are a restricted total-by-construction sublanguage

**Status:** accepted

Only functions marked `measure` may appear inside constraint predicates, and a
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

- The `static`/`dynamic` verification-mode split is about *solver* decidability,
  never about measure totality — every type-level function is guaranteed total.
- Arbitrary iterative/general-recursive pure algorithms cannot be used directly
  in constraints; restructure into structural recursion, or use a `dynamic`
  predicate / `assume` boundary.
- Structural recursion covers the common measures (len, height, sum, contains,
  sorted, balanced, …), so the restriction retains practical usefulness.
- `pure` and `measure` are distinct keywords with distinct guarantees.
