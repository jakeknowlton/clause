# Measures are a restricted total sublanguage (well-founded recursion)

**Status:** accepted

Which functions may appear inside a constraint predicate depends on the
constraint's **verification mode**: a **`static`** predicate may use only
`measure` functions (the solver reflects them as axioms); a **`dynamic`**
predicate may use any `pure` function (executed at runtime, never reasoned
about — a non-terminating dynamic check is a hang, not unsoundness). Predicates
are restricted to runtime-evaluable forms (bounded quantifiers only).

A `measure` is restricted to a sublanguage that is **total**:

- **side-effect-free** (a subset of `pure`);
- **well-founded recursion** — every recursive call (including across a
  mutual-recursion group) must strictly decrease a metric under a well-founded
  order. The metric is **inferred by default** and overridable with a
  `decreases` clause;
- **total operations only** — no trapping ops; partial operations (indexing,
  division) require their guards to be provable at the use site.

## Termination: well-founded, not "by form"

An earlier framing claimed termination "by form" via *structural* recursion
alone. That is too weak for Clause: the solver reasons over **unbounded
mathematical `Int`** (ADR-0006), which is not an inductive type, and the core
sequence types (`List`/`String`/`Slice`) are **array-backed**, not cons-cells —
so numeric recursion (`pow`, `gcd`) and length-indexed recursion are *not*
structural descent. We therefore adopt **well-founded recursion with a default
metric and an optional `decreases` clause**, the consensus design of SMT-backed
verifiers that reflect functions as axioms — Liquid Haskell (`/ [e]` metrics),
Dafny and F\* (`decreases` clauses); cf. Lean 4 and ACL2.

**Default-metric catalog:**

| Argument kind | Default metric |
|---|---|
| inductive / `enum` | structural sub-term (a strict sub-component is smaller) |
| `Nat` (`Int where it >= 0`) | the value, ordered by `<`, bounded below by 0 |
| sequence (`List`/`String`/`Slice`/`[T,N]`) | `len`/`byteLen`, ordered by `<` |
| multiple arguments | lexicographic tuple in parameter order |

The default guess is the first parameter that strictly decreases; if none is
obvious, the compiler **requires** an explicit `decreases`. **Mutual recursion**
is allowed within a recursion SCC: structural sub-term ordering handles
mutually-recursive ASTs for free (the canonical self-hosting case, ADR-0011),
while mutual *numeric* recursion requires the group to share a well-founded order
via `decreases`.

## Totality is the one sound-by-rejection obligation

Unlike every other constraint in Clause — each *trichotomous* with a visible
runtime fallback (ADR-0001: Unknown ⇒ runtime check, never a compile error) — a
measure's **decrease obligation has no runtime fallback**. Totality is what
makes a recursive measure's defining equations a *consistent* axiom set, and
those axioms are consumed at **compile time** to prove *other* code; a measure
that is only "probably total" is worthless and actively dangerous (it would let
the solver "prove" false things far from the measure's own definition). The
per-call decrease VC must therefore be **Proven or it is a compile error**, and
is discharged in the **decidable linear/structural core only** (never over the
measures themselves, which would be circular). Measures are thus the single
corner of Clause that is sound-by-rejection — Dafny/F\*/LH reject here too; none
offers a runtime fallback for termination.

The proven/runtime-guaranteed dichotomy is still preserved at the language level:
a fact you cannot *prove* total you can still *runtime-check* by dropping to a
`dynamic` predicate over the `pure` version of the function — you simply may not
feed it to the solver as a static axiom.

## Why

The solver reflects a recursive measure's defining equations as axioms. A
non-terminating (non-total) measure yields an inconsistent axiom set, from which
the solver can "prove" anything — i.e. it is *unsound*, not merely slow. Proving
termination of *arbitrary* functions is undecidable (the halting problem), so
instead of a general termination prover we restrict measures to well-founded
recursion and discharge a single local decrease VC per call site — almost always
a trivial linear or structural fact.

Note this concerns *measure* termination (soundness of the axioms). *Solver*
termination (decidability of a given query) is a separate matter, handled by the
ADR-0001 timeout ⇒ Unknown ⇒ runtime-check path.

## Consequences

- `measure` totality is **mandatory and compile-time-proven**: `static`
  predicates get sound axioms; `dynamic` predicates need only purity.
- The default-metric guess will sometimes fail; the user then writes an explicit
  `decreases`. This is a known ergonomic cost (cf. Lean's well-founded goals),
  not a soundness risk.
- Arbitrary iterative / general-recursive algorithms still cannot be measures;
  restructure into well-founded recursion or use a `dynamic` predicate.
- Well-founded recursion covers the common measures (len, height, sum, sorted,
  balanced, AST size/depth) and numeric measures (pow, gcd), retaining practical
  usefulness.
- `pure` and `measure` remain distinct keywords with distinct guarantees.
- Adds one reserved keyword, `decreases` (valid only in a measure signature).
- A measure is also an ordinary callable `pure` function (not predicate-only); its
  signature uses machine types (`-> U64`, …) — `Nat`/`Int` are the solver's view,
  never written.
