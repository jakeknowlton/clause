# Memory management: ARC with reuse analysis, not tracing GC

**Status:** accepted (supersedes the "garbage-collected" framing in the original
vision)

Clause manages memory with **automatic reference counting (ARC) plus reuse
analysis** (the Koka *Perceus* / Lean-4 model), **non-moving** and
**deterministic** — not a tracing garbage collector. The compiler inserts
retain/release automatically (no lifetime annotations, no borrow checker); only
reference-typed values (`class`, boxed interface-values, closures, growable
collections) are refcounted — value types (`struct`, the numeric tower) are
never touched. When a value's refcount is 1, reuse analysis reclaims its
allocation in place, compiling functional updates into in-place mutation.

## Why (fit to Clause's existing decisions)

- **C interop "functions well":** non-moving ⇒ C can hold a Clause pointer
  safely (no pinning); deterministic destruction ⇒ a Clause object wrapping a C
  resource frees it promptly (clean RAII across FFI), which tracing GC
  finalizers do badly.
- **Leverages immutability-by-default:** Perceus reuse turns `fix`-heavy
  functional code into in-place mutation — immutability becomes a *performance*
  feature, not just a safety one.
- **Not Rust:** all retain/release is inferred; no lifetimes, no borrows.
- No GC pauses; predictable footprint.

## Considered options

- **Tracing GC:** best throughput, cycle-free, but moving/pinning + safepoints
  complicate C interop and finalization is nondeterministic.
- **Plain ARC (Swift):** good, but no in-place reuse.
- **Regions, linear types, constraint-proven ownership:** kept as future
  augmentations, not the base scheme.

## Consequences

- **Cycles are not auto-collected** — broken with explicit `weak` references;
  an optional cycle-collector is a possible later addition.
- Implementing reuse analysis is real work (mitigated by prior art: Koka,
  Lean 4).
- Opens two Clause-only future doors: regions/arenas as an opt-in allocator, and
  **verifier-assisted refcount elision** (the solver proves a retain/release
  pair unnecessary).
