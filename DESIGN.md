# Clause — Language Design

A general-purpose, self-hosted language compiled via LLVM, memory-managed by
automatic reference counting with reuse analysis, whose distinguishing feature is
**constraints**: refinement predicates on values that the compiler verifies,
proving as many as possible at compile time and enforcing the rest at runtime.

This document is the narrative overview. Precise decisions and their rationales
live in [`docs/adr/`](docs/adr/); the canonical vocabulary lives in
[`CONTEXT.md`](CONTEXT.md). ADR references are given inline as `[ADR-N]`.

---

## 1. The constraint system (the core)

A **constraint** is a logical predicate a value must satisfy (`x > 0`,
`i < len(xs)`). It refines a base type into a precise type, written canonically
`{ v: T | predicate }` and, on declarations, with `where` sugar:

```clause
fix x: I32 where x > 0 = 5;
type Positive = I32 where it > 0;     // 'it' binds the value in anonymous positions
```

**Two-layer type system.** A type has a **base type** (`I32`, a class, …) checked
by ordinary Hindley–Milner unification, and a **refinement predicate** checked by
logical *entailment* via a solver. Refinements never interfere with inference:
`z = x + y` gives `z : { v | v == x + y }`, so refinements *propagate* rather than
unify.

**Enforcement is a trichotomy** [ADR-0001]:
- **Proven** → no runtime cost.
- **Refuted** (a concrete, reachable counterexample) → **compile error**.
- **Unknown** (timeout/undecidable) → a **visible** runtime check that traps.

So every constraint holds statically *or* via a runtime trap — never silently
unchecked — and the developer is always told where the prover gave up.

**Verification modes** (a type may carry several conjoined constraints, each with
its own mode):
- `static` (default) — try to prove; fall back to a runtime check if Unknown.
  May use only `measure` functions.
- `dynamic` — skip the solver, always check at runtime; may use any `pure`
  function.

There are only these two modes: every constraint is either *proven* or
*runtime-guaranteed* — nothing is ever merely trusted (which is why predicates
are kept runtime-evaluable: bounded quantifiers only). Once any checked
constraint holds, downstream code may treat it as a **proven fact** and feed it
to the solver. Modes are part of the type's representation but do not affect
subtyping, which uses only the logical predicate.

**Predicate language.** A decidable core (boolean logic, equality, linear integer
arithmetic, arrays/sequences) extended by **measures** [ADR-0003]: functions
marked `measure`, restricted to a **total-by-construction** sublanguage
(side-effect-free, structural recursion only, total operations only). Measures are
the *only* functions usable in predicates; the `static`/`dynamic` split concerns
*solver* decidability, never measure totality. (`pure` is a separate, weaker
modifier — side-effect-free but not necessarily total, and not usable in
predicates.)

**Mutation** [ADR-0002]. Clause is ARC-managed with free aliasing and
deliberately has **no ownership/borrow analysis**. Soundness without one:
- A mutable binding *may* carry a constraint, re-checked at every assignment.
- A *dependent* predicate (referencing values other than the one constrained) may
  only reference **immutable** values/fields — so no mutation can ever invalidate
  a dependent proof, eliminating aliasing analysis by construction.

`fix` (immutable) is the idiomatic default and the prover-friendly form; `let` is
mutable.

## 2. Functions and contracts

Contracts *are* refinements on parameter/return types — one mechanism, the unified
`where`:

```clause
fun get(xs: [I32], i: U64) -> I32
    where i < len(xs)             // precondition (relational, function-level)
{ ... }

fun abs(x: I32) -> I32 where it >= 0 { ... }   // postcondition: 'it' = result
```

Verification is **modular**: a function is checked once against its signature
(body assumes preconditions, must prove postconditions); call sites check against
the signature only and assume postconditions. Functions are first-class; their
types carry their contracts (contravariant preconditions / covariant
postconditions). **Closures** capture by **immutable snapshot** and are
ARC-managed; sharing mutable state requires an explicit `class` reference.

## 3. The type system

Four kinds of user-defined type:
- **`struct`** — value type, all fields public, no constructors (struct-literal
  init); may carry an **invariant** checked at each construction/write site (sound
  via value semantics). The ABI interchange type for C.
- **`class`** — reference type, ARC-managed, fields private by default; may carry
  an **invariant** established by each `init` and maintained at public-method
  boundaries — an implicit pre/postcondition of public methods, enforced at every
  public-method call (`self`-calls included) to stay sound under reentrancy
  [ADR-0016]. Constructors are unnamed `init`s dispatched by base-type signature.
- **`enum`** — sum/tagged-union with pattern matching; basis for `Option`/`Result`;
  exhaustiveness is a proof.
- **`interface`** — the *only* source of subtype polymorphism. **No implementation
  inheritance** [ADR-0004], keeping every invariant local and unweakenable.

**No null** — references are always valid; absence is `Option<T>`.

**Generics** are **monomorphized** [ADR-0007], bounded by interfaces; using an
interface *as a value* is a boxed, vtable-dispatched existential. v1 supports
container-level refinements over generics; value-level refinement over an abstract
`T` is a later extension.

**Overloading & conversions** [ADR-0005]: dispatch is on **base-type signatures
with refinements erased** (no two overloads share one); **no implicit
conversions** — all casts are explicit and carry their obligations.

`match` is exhaustiveness-checked with `where` guards. Operators desugar to
built-in interfaces (`Add`, `Ord`, `Index`, …); `for` desugars to an `Iterator`
interface, with `Range` as the solver's loop-bound fact source. Interface
coherence uses the orphan rule; `Eq`/`Ord`/`Hash`/`Show` are structurally
derivable.

## 4. Numbers and overflow

Machine integers are **mathematical integers refined by a range** [ADR-0006]:
`I32 ≡ { v: Int | -2^31 <= v < 2^31 }`. Arithmetic is exact in the logic; overflow
is just the target type's range refinement re-imposed, discharged by the usual
trichotomy (proven away in hot loops, trapped otherwise). Explicit `+%`/`-%`/`*%`
opt into wrapping. Silent wraparound is eliminated by default.

The solver's domain is **linear integer arithmetic over `Int`** — which keeps
specs width-agnostic and overflow separable — with a **bitvector bridge** for the
bit-level operators (`&`/`|`/`^`/`~`/`<<`/`>>`, `+%`/`-%`/`*%`) that LIA cannot
express. `Int`/`Nat` are **logical, compile-time-only** types with no runtime
representation; every runtime integer is fixed-width, and arbitrary precision is
an opt-in `BigInt` standard-library type [ADR-0006]. When a measure-bearing
constraint is left as a runtime check, the measure is evaluated in trapping
machine arithmetic — never bignum, never wrapping [ADR-0014]. Floating-point
(`F16`/`F32`/`F64`) sits **outside** this static core in v1 — not range-refined
and IEEE-total (inf/NaN, no trap), so float constraints are runtime-checked
[ADR-0006].

## 5. Errors: two channels

[ADR-0008] **Recoverable, world-dependent** failures are **`Result<T, E>`** values
with `?`-propagation — no exceptions (the solver never models non-local exits).
**Proof failures** (failed runtime checks, overflow)
**trap/abort**, non-catchably. The dividing line: *preventable-by-proof ⇒
constraint/trap* (`xs[i]`, integer `a/b`, casts); *world-dependent ⇒ Result* (I/O,
parsing). Clause thus replaces much `Option`/`Result` ceremony with proofs.

## 6. Memory

[ADR-0009] **Automatic reference counting with reuse analysis** (Perceus/Lean-4
style), **non-moving** and **deterministic** — not tracing GC. Only reference
types are refcounted; value types are never touched. Reuse turns `fix`-heavy
functional updates into in-place mutation (immutability becomes a *performance*
feature). Non-moving + deterministic gives clean C interop and RAII-style cleanup
of C resources. Cycles are broken with explicit `weak` references. Future doors:
regions/arenas, and verifier-assisted refcount elision.

## 7. C interoperability

[ADR-0010] A quarantined trust boundary keeps the safe core safe:
- **`Ptr<T>`** — raw, unmanaged, nullable pointer, distinct from a managed
  reference.
- **`unchecked`** region — the only place raw-pointer deref and `extern` C calls
  are allowed; all unsafety is named and greppable. Constraints on C-returned
  data are enforced with ordinary `dynamic` runtime checks.
- Layout-compatible **structs** are the ABI interchange; `extern` declares C
  signatures.

## 8. Modules, packaging, the standard library

A **module** is a file (with optional nested `module` blocks); modules form a tree
addressed by `::`. A **package** is the build/distribution unit with a manifest.
**Three-tier visibility:** `public` / `internal` / private-default; `import` for
imports. Exported signatures carry their constraints (part of the API contract). The library splits into **`core`** (intrinsic, compiler-shipped, freestanding =
no-OS) and **`std`** (hosted, written in Clause); only **`core::prelude`** is
auto-imported, while `std` (including I/O) is always an explicit `import`
[ADR-0017]. A **tiny intrinsic core** is built in; the rest is written in Clause,
dogfooding self-hosting and constraints.
Strings are immutable UTF-8, ARC-managed, byte-indexed with constraints. Slices
are ARC-retaining views — **no lifetimes**.

## 9. Compiler & bootstrap

[ADR-0011] **Staged self-hosting:** a host-language bootstrap compiler (Stage 0) →
rewrite in Clause (Stage 1) → self-host (Stage 2). Pipeline: lex → parse → resolve
→ base-type inference → **modular refinement verification (pre-monomorphization)**
→ monomorphize → **typed MIR** (match compilation, runtime-check insertion,
ARC/reuse) → LLVM IR → native. The **solver is layered and low-coupled**: a native
fast-path discharger for common obligations, with external SMT (Z3 first) as a
pluggable backend behind a verification-condition abstraction, so more solving can
move native over time. The solver is a compiler dependency, never shipped in
binaries.

## 10. Concurrency

[ADR-0012] Full design **deferred post-v1**, but the **direction is fixed**:
message-passing / actor-style isolation with **no shared mutable state** —
race-free without a borrow checker, falling out of the no-aliasing and
immutability decisions. v1 is single-threaded with non-atomic refcounts; the
object model anticipates atomic/handoff refcounting for shared immutable data.

---

## Open frontiers (deliberately deferred)

- Value-level refinements over abstract generic type parameters (needs
  interface-published measures) [ADR-0007].
- Fully-general higher-order contracts (beyond concrete function types).
- Verifier-assisted refcount elision; regions/arenas [ADR-0009].
- A backup cycle-collector for ARC.
- The concrete concurrency model (actors vs CSP vs async) [ADR-0012].
- Auto-binding generation from C headers [ADR-0010].
- Optional loop-`invariant` annotations — v1 proves index bounds via `Range`
  facts and degrades loop-carried properties to runtime checks (the trichotomy).
- Purity-polymorphism for higher-order functions (v1 has a one-bit purity arrow;
  "pure iff the callback is pure" needs effect variables) [ADR-0015].
- Faithful floating-point reasoning via the SMT FP theory (QF_FP); v1 keeps
  floats outside the static core [ADR-0006].
- String interpolation in string literals (v1 formats via the `Show` interface +
  `+` / `StringBuilder`; there are no `printf`-style variadics).

## Decision index

- ADR-0001 — Hybrid constraint enforcement (trichotomy, visible runtime fallback)
- ADR-0002 — No aliasing analysis (mutable constraints, immutable-only dependence)
- ADR-0003 — Measures are a restricted total-by-construction sublanguage
- ADR-0004 — No implementation inheritance; polymorphism via interfaces
- ADR-0005 — Explicit conversions; base-type overload dispatch
- ADR-0006 — Integers as range-refined mathematical integers
- ADR-0007 — Monomorphized generics; interface bounds; boxed interface-values
- ADR-0008 — Two-channel error model (Result vs trap)
- ADR-0009 — ARC with reuse analysis, not tracing GC
- ADR-0010 — C FFI boundary (`Ptr<T>`, `unchecked`, structs as ABI)
- ADR-0011 — Compiler architecture (staged bootstrap, typed MIR, layered solver)
- ADR-0012 — Concurrency direction (message-passing, deferred)
- ADR-0013 — No `assume` mode (every constraint proven or runtime-checked)
- ADR-0014 — Measure runtime-evaluation in trapping machine arithmetic (no bignum)
- ADR-0015 — `pure` is a checked, transitively-verified effect annotation
- ADR-0016 — Class-invariant boundary discipline (implicit pre/post of public methods)
- ADR-0017 — Package architecture (`core`/`std`; only `core::prelude` auto-imported)
- ADR-0018 — Integer operator semantics (truncating div/mod, shift rules)
