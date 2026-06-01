# `pure` is a checked, transitively-verified effect annotation

**Status:** accepted

`pure` marks a function (or method) as **observably effect-free and
deterministic-within-a-run**. It is a *checked* annotation — declared, then
verified by the compiler — not merely inferred, so it is part of a public API's
contract (like `measure`, of which it is a superset: `measure ⊂ pure`).

## What `pure` guarantees, and how it is checked

A function is `pure` iff, transitively:

- it performs **no I/O and enters no `unchecked`/`extern` (FFI) region**;
- it **calls only `pure` functions/methods**;
- it **mutates only value-typed locals** (`struct`/numeric `let`s) — never
  reference (`class`) state;
- it **reads only immutable state** — `fix` fields/bindings, never a mutable
  (`let`) field — which is what upgrades effect-freedom to determinism.

This is a **transitive, syntactic, modular** check requiring **no escape or
aliasing analysis** (ADR-0002): mutation of a `class` flows only through methods,
so a mutating method is simply non-`pure` and a `pure` function cannot call it;
value-typed locals don't alias, so mutating them is unobservable; and reading
only `fix` state mirrors ADR-0002's "dependent predicates reference only
immutable values," giving within-run determinism structurally (no I/O, no FFI, no
mutable reads ⇒ same inputs, same result).

## Purity on function types; polymorphism deferred

Higher-order purity requires the **function type to carry a purity bit**:
`pure (A) -> B` is a subtype of `(A) -> B` (purity is a guarantee that may be
forgotten, never invented). A higher-order function that is itself `pure`
*requires* a `pure` callback —
`pure fun map<T, U>(xs: List<T>, f: pure (T) -> U) -> List<U>`.

Full **purity-polymorphism** ("`map` is pure exactly when `f` is," needing effect
variables) is **deferred**: in v1 a HOF either requires a pure callback (and is
pure) or does not (and is not). Standard higher-order combinators ship `pure`
over `pure` callbacks — the dominant case; an effectful traversal is an ordinary
`for` loop. Monomorphization (ADR-0007) may additionally *infer* a tighter
per-instantiation purity for optimization, but inferred purity is never part of a
public contract.

## Why `pure` exists (it is weaker than `measure`)

`pure` is *not* usable in `static` predicates (those admit only `measure`s, whose
totality yields sound axioms — ADR-0003); a `pure` function may be non-total. Its
jobs are: (1) the function vocabulary for **`dynamic`** predicates (executed,
never reasoned about, so effect-freedom — not totality — is what they need); and
(2) licensing **optimizations** (CSE, reordering, dead-call elimination,
verifier-assisted refcount elision — ADR-0009), which is exactly why the chosen
meaning is determinism, not merely effect-freedom.

## Consequences

- `pure` is verified, not assumed; a violated `pure` is a compile error.
- Function types gain a purity bit (`pure (A) -> B`); a grammar addition.
- Reading a mutable (`let`) field from a `pure` function is rejected — push the
  read to the caller, or make the field `fix`.
- Purity-polymorphic higher-order signatures are a later extension.
