# Clause — Context & Glossary

The shared language of the Clause project. This file is a **glossary only** — no
implementation details, no specs, no decisions. Decisions live in `docs/adr/`.

Clause is a (planned) general-purpose, self-hosted language compiled via LLVM,
managed by **automatic reference counting with reuse analysis** (not tracing
GC), whose distinguishing feature is **constraints**: refinement predicates on
values that the compiler verifies, proving as many as possible at compile time.

Source files use the **`.cla`** extension (chosen over `.cl`/`.cls`, which
collide with OpenCL/Common Lisp and LaTeX/Apex respectively).

---

## Glossary

### Constraint
A logical predicate attached to a value that the value must satisfy, e.g.
`x > 0` or `i in 0..len`. A constraint refines a base type into a more precise
type. The compiler is responsible for ensuring every value abides by its
constraints — proving them at compile time where possible, and otherwise
falling back to a runtime check (mechanism TBD).

*Not to be confused with* a **type equation** (see below), which is an internal
unification fact, not a user-facing feature.

### struct
A **value type**: no identity, copied on assignment, never aliased, may live
inline. Used for plain data, performance, and mapping C structs (FFI). May carry
an **invariant**. Its copy semantics make invariants trivially sound (no
aliasing).

### class
A **reference type**: heap-allocated, garbage-collected, has identity, freely
aliased. Holds methods and an **invariant**. The "objects instantiated from
class constraints" of Clause's vision. No implementation inheritance (ADR-0004).

### Ptr<T> / unchecked / extern
The C-interop vocabulary. **`Ptr<T>`** is a raw, **unmanaged** pointer (not
refcounted, may be null, carries no assumed constraints) — distinct from a
Clause reference (which is managed, non-null, always valid). **`unchecked`**
demarcates the trust region: the only place raw-pointer dereference and `extern`
C calls are permitted, so the rest of the language stays provably safe and every
unsafe act is greppable. Constraints on C-returned data are enforced with
ordinary `dynamic` runtime checks. **`extern`** declares an imported C
function signature. Layout-compatible **`struct`s** are the ABI interchange
currency; `class`es (with headers/refcounts) are not passed to C directly. See
ADR-0010.

### Closure / function type
Functions are first-class values; a function type is `(A, B) -> C` and carries
the param/return **refinements** as part of the type (with contravariant
preconditions / covariant postconditions when one function type is used where
another is expected; fully-general higher-order contracts are a later
extension). **Closures** capture their environment by **immutable snapshot**
(copy value types, retain references) — never by mutable reference, so no mutable
state is aliased across the closure boundary (consistent with ADR-0002). Sharing
mutable state into a closure requires passing an explicit reference (`class`).
Closure objects are heap-allocated and ARC-managed. Lambda syntax uses a fat
arrow: `(x, y) => x + y` (parens optional for a single param), keeping `->`
exclusively for function *types*.

### Operators / Iterator / coherence / deriving
**Operators** are sugar for built-in **interfaces** (`+` → `Add::add`, etc.); no
ad-hoc operator definitions. **`for x in coll`** desugars to an **`Iterator`**
interface (`next() -> Option<T>`); the **`Range`** type (`0..n`) is the canonical
iterator and the solver's source of loop-bound facts (`i in 0..n ⟹ 0 <= i < n`),
making range `for`-loops a prime site for proving away bounds checks.
**Coherence:** an `impl` of interface `I` for type `T` must live in `I`'s or
`T`'s package (the orphan rule). **Deriving:** `Eq`, `Ord`, `Hash`, `Show` can be
auto-derived structurally.

### match
The matching expression (arms use `=>`). An arm-left is either a **structural
pattern** (variant/tuple/struct/slice/literal/range, with sub-bindings) or a
**condition arm** — a `Bool` expression mentioning **`it`** (the scrutinee).
There are **no `where`/`if` guards**; the only catch-all is `_`, and the
scrutinee is reached via `it`. **Exhaustiveness is a proof:** structural matches
by closed-variant coverage, condition matches when the solver proves the arms
total (else a `_` is required). Within a structural arm the solver knows the
concrete variant. No `@`-bindings.

### init
An unnamed class constructor. A class may declare several `init`s; they are
distinguished by their **base-type signature** (the ordered list of parameter
base types, with refinements *erased*). No two `init`s — and more generally no
two overloads of any function/method — may share a base-type signature.
Parameters may still carry refinements as preconditions; those just don't
participate in dispatch. Called as `Account()`, `Account(100)`, etc. Each `init`
must definitely-assign all fields and establishes the invariant on return.

### enum
A **sum / tagged-union type** with pattern matching. The basis for `Option`,
`Result`, state machines, etc. The solver reasons about its closed set of
variants (exhaustiveness becomes a proof).

### Option / no-null
Clause has **no null**: references are always valid and dereferenceable.
Absence is expressed explicitly by `Option<T>` (`Some(T) | None`), consumed by
pattern matching. Non-nullness is therefore *structural*, never a `where v !=
null` refinement.

### interface
Abstract behavior implemented by structs/classes. The *only* source of subtype
polymorphism in Clause (there is no implementation inheritance). Used two ways:
as a **bound** on a generic type parameter (`<T: Ord>`, static, monomorphized),
or as a **value** (`shape: Drawable`), which is a boxed, vtable-dispatched
existential (dynamic dispatch — the explicit opt-in to indirection).

### Generics / monomorphization
Parametric polymorphism (`List<T>`, `Map<K,V>`). Compiled by **monomorphization**
— a specialized copy per concrete instantiation — which keeps value types
unboxed and exposes concrete refinements to the solver per instantiation. See
ADR-0007.

### Invariant
A constraint (`where` predicate over fields) attached to a `struct` or `class`
that must hold whenever the value is observable. The *checkpoints* where it is
verified differ by type kind:
- **struct** (value, open, all fields public): checked at every construction
  literal and every field-write site — sound because value semantics give no
  aliasing. Relational invariants that can't be maintained field-by-field are
  preserved by functional whole-value update.
- **class** (reference, encapsulated): *established* by a constructor (its
  implicit postcondition) and *maintained* across public-method boundaries
  (assume-on-entry / re-establish-on-exit; may break temporarily inside; private
  helpers exempt). Mutation flows through methods.

A C-interop struct simply declares no invariant and is plain data.

### Refinement type
The canonical form of a constrained type, written `{ v: BaseType | predicate }`
(set-builder). The bound variable (`v` here) names the value the predicate
constrains.

### where clause
Surface sugar for a refinement type on a declaration: `let x: I32 where x > 0`.
On a named declaration the declared name (`x`) is the binder; in anonymous
positions the pronoun `it` binds the value (`type Positive = I32 where it > 0`).
Desugars to the canonical refinement type. A verification mode keyword may
precede the predicate (`where dynamic isPrime(p)`).

### Module / package / visibility
A **module** is a source file (the implicit default); nested `module Name { }`
blocks may group/sub-scope within a file. Modules form a tree by directory/file,
addressed with `::` (e.g. `json::parser`). A **package** is the build &
distribution unit — a module tree with a manifest (name, version, dependencies,
public root); exported generic functions ship as IR for importers to specialize
(ADR-0007). **Visibility is three-tier:** `public` (visible to importing
packages), `internal` (visible within the package), and private (the default —
this module/file only). Imports use `import`. An exported function's refinements
are part of its public contract (strengthening a precondition / weakening a
postcondition is a breaking change).

### Verification mode
How a single constraint is checked. A type may carry several constraints
(conjoined), each with its own mode:
- **static** (default) — proven at compile time where possible; Refuted ⇒
  compile error, Unknown ⇒ runtime check.
- **dynamic** — solver is skipped entirely; always a runtime check. Lets the
  predicate use any `pure` function at the cost of a guaranteed runtime check.

There are only these two modes: every constraint is either *proven* (`static`)
or *runtime-guaranteed* (`dynamic`) — Clause has no "trusted/unchecked" fact, so
nothing is ever taken on faith. (This is why the predicate language is kept
runtime-evaluable: bounded quantifiers only.)

Once any checked constraint (static-proven, or dynamic/static-checked at runtime
past its check point) holds, downstream code may treat it as a **proven fact**
and feed it to the solver as an assumption.

Modes **are** carried in the type (a refinement is base + a list of moded
constraints, written the same way in `where`-form and set-builder). But
**subtyping / precondition-satisfaction depends only on the logical predicates**
— a value of `{v | p}` satisfies a requirement for `{p' | p}` regardless of
either side's modes. Modes instead drive: **well-formedness** (`static` ⇒ only
`measure` functions; `dynamic` ⇒ any `pure` function) and the **code emitted** at
establishment/discharge sites. See [[refinement-type]].

### Base type
The unrefined type of a value — `I32`, `Bool`, `[T, N]`, a class, etc. Base
types are inferred/checked by ordinary Hindley–Milner unification. Every value
has exactly one base type.

### String / collections / slice
**`String`** is immutable, UTF-8, ARC-managed (O(1) sharing); its byte-length is
a `measure`. Integer indexing `s[i]` means *bytes* and requires `i < byteLen(s)`
(a constraint); code points come from iterators (no O(1) char indexing) as
**`Char`** values (a primitive Unicode-scalar type with `'a'`/`'\u{…}'`
literals). A mutable **`StringBuilder`** handles construction. C strings cross
only inside `unchecked`. Growable **`List<T>` / `Map<K,V>` / `Set<T>`** are
standard-library types written in Clause, ARC-managed, exposing `len` and
contract-carrying APIs (`pop` requires non-empty, etc.). A **`Slice<T>`** is an
ARC-retaining view with its own length — because it retains its backing storage,
Clause needs **no lifetimes**.

### Standard library
A **tiny intrinsic core** (primitives, `[T,N]`, `Option`, `Result`, `String`,
the `Ptr`/FFI bits) is built into the compiler; everything else is **written in
Clause**, dogfooding self-hosting and the constraint system.

### Int (mathematical integer)
The unbounded mathematical integer the solver reasons over. The machine integer
types (`I8`…`I64`, `U8`…`U64`) are defined as `Int` refined by a range, so
arithmetic is exact in the logic and overflow is just the target type's range
refinement (see ADR-0006). Wrapping (`+%`, `-%`, `*%`) opts into modular
semantics explicitly.

### Refinement predicate
The logical proposition half of a constraint — the `{ v | P(v) }` part. Checked
by *logical entailment* (an SMT solver), never by unification. A value's full
type is `{ v: BaseType | RefinementPredicate }`.

### Verification condition (VC) / Solver
A **verification condition** is the logical obligation emitted for a constraint
that must be discharged (e.g. `ctx ⇒ 0 <= x < 256`). The **Solver** is a layered
abstraction behind which VCs are discharged: a **native fast-path discharger**
(constant folding, interval/range reasoning, syntactic implication) handles the
common easy obligations without external help, and a **pluggable external SMT
backend** (Z3 first) handles the rest. The compiler is deliberately
low-coupled to Z3 so more solving can move native over time. See ADR-0011.

### Type equation
**(Internal term — renamed to avoid collision.)** An equality between base
types produced during inference and resolved by the unifier (what the current
`type_checker.rs` calls a "constraint"). Has nothing to do with the user-facing
**Constraint** feature.

### Precondition / Postcondition
A **precondition** is a constraint a function's caller must establish before the
call; written either inline on a parameter (`x: I32 where x > 0`, sugar for a
single-param case) or in the **pre-`->` `where` zone** of the signature, which
may reference all parameters and (in a method) `self` — covering relational and
receiver-state preconditions. A **postcondition** is a constraint the body must
establish; written in the **post-return `where` zone**, where `it` binds the
result and may reference any (immutable) parameter. Position disambiguates:
before `->` ⇒ precondition; after the return type ⇒ postcondition.

### Modular verification
Each function is verified once against its signature: the body is checked
*assuming* its preconditions and *must prove* its postconditions. Call sites are
checked only against the signature — they prove the preconditions and may then
*assume* the postconditions — and never re-analyze the body. This is what keeps
whole-program constraint checking tractable.

### Measure
A function usable *inside* a constraint predicate, marked with the `measure`
keyword. A measure belongs to a restricted **total-by-construction**
sublanguage: side-effect-free, structural recursion only (no general loops or
recursion), and total operations only (no trapping ops; partial operations like
indexing/division require their guards to be provable). Measures are the *only*
functions that may appear in predicates. Termination is guaranteed by the
sublanguage's *form*, not by a termination prover. See ADR-0003.

### pure
A modifier marking a general function as side-effect-free (deterministic, no
I/O, no non-local mutation). Distinct from `measure`: a `pure` function is *not*
necessarily total and may *not* appear in predicates. (Purity is still useful
for optimization and reasoning.)

### ARC / reuse / weak
Clause's memory management. **ARC** = automatic reference counting: the compiler
inserts retain/release on reference-typed (`class`, boxed, closure, growable
collection) values; value types (`struct`, numerics) are never refcounted.
**Reuse analysis** (Perceus / Lean-4 style) inserts those operations precisely
and, when a value's count is 1, reuses its allocation in place — turning
functional updates into in-place mutation (leans on immutability-by-default).
The collector is **non-moving** (C may hold pointers) and **deterministic**
(prompt RAII-style cleanup, good for C resources). Cycles are not auto-collected;
a **`weak`** reference breaks them, with an optional cycle-collector as a later
backup. See ADR-0009.

### Result / trap
The two failure channels (ADR-0008). **`Result<T, E>`** is a value-level enum
for recoverable, world-dependent errors (file I/O, parsing), propagated with a
`?`-style operator; there are no exceptions. A **trap** is the unrecoverable,
non-catchable abort taken when a *proof failure* occurs at runtime (a failed
`dynamic`/fallback constraint check, or an overflow). Dividing line:
preventable-by-proof ⇒ constraint/trap; world-dependent ⇒ `Result`.

### fix / let
`fix` introduces an immutable binding; `let` introduces a mutable binding.
(Spellings carried over from the original `main` design.) Immutability is the
**idiomatic default** — `fix` is presented first in all docs/examples and is the
prover-friendly form — but it is a *cultural* default, not a syntactic one: both
forms take a keyword. `fix x = 5` reads as "fix the value of x," matching the
mathematical idiom.
