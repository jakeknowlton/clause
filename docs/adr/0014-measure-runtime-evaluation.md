# Measure runtime-evaluation: trapping machine arithmetic, no bignum

**Status:** accepted

A `static` constraint may mention `measure`s and may, per ADR-0001, fall back to
a **runtime check** when the solver returns Unknown. That runtime check must
*evaluate the measure* — but measures are specified over logical `Int`/`Nat`
(ADR-0006), which has no runtime representation. This ADR fixes what the fallback
evaluates on.

## Decision

- A measure's runtime-fallback evaluation runs in **fixed-width machine
  arithmetic** — the machine types in the measure's signature, or the widest
  native type where the logical result is conceptually unbounded. **No bignum
  runtime is linked**, consistent with ADR-0011 (the solver — and now
  arbitrary-precision arithmetic — never ships inside compiled programs).
- Measure evaluation **never wraps**: an overflow during fallback evaluation
  **traps** (ADR-0008 fail-fast). Because it never wraps, the fallback check is
  never *unsound* — "didn't trap ⇒ the constraint held" survives.
- Eval-overflow is **not** required to be proven absent. It is simply one more
  way a runtime check can trap. For the common measures (`len`, `height`,
  `sorted`, `contains`, `balanced`) the result is physically bounded by the
  address space, so the trap is unreachable in practice; only a genuinely
  unbounded accumulator (e.g. `factorial(n)` for large `n`) reached on a
  *fallback* path can hit it.

## Why

Measure **totality** (termination + definedness, ADR-0003) is proven in the
*logic* over unbounded `Int`, where overflow does not exist — so `height` of a
finite tree is finite and total with no width obligation. Overflow is purely a
*machine-evaluation* artifact, arising only on the runtime-fallback path, which
is already a path that may trap. Requiring measures to be provably overflow-free
at machine width would reject ordinary measures like `height(t) -> U64` (you cannot
prove a tree has < 2⁶⁴ levels from its type); shipping a bignum to evaluate them
faithfully would violate the runtime-footprint story (ADR-0009/0011). Trapping on
the physically-unreachable tail is the honest middle path.

## Consequences

- The runtime stays bignum-free; measure evaluation is plain machine code.
- A measure-bearing predicate left as a runtime fallback can, in principle, trap
  *inside* the measure on a pathological input — a trap, never a silent wrong
  answer.
- Measures used only in *proven* predicates are never evaluated and never face
  this (pure compile-time logic).
