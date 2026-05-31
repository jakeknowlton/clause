# C FFI: unmanaged `Ptr<T>`, an `unchecked` trust region, structs as ABI

**Status:** accepted

C interop is built around a quarantined trust boundary so the safe core stays
provably safe:

- **`Ptr<T>`** — a raw, unmanaged pointer (not refcounted, nullable, no assumed
  constraints), distinct from a managed Clause reference (non-null, always
  valid). C `extern` signatures traffic in `Ptr<T>`; bridging to/from a managed
  reference is an explicit act.
- **`unchecked` region** — the *only* place raw-pointer dereference and `extern`
  calls are allowed. All unsafety is named, contained, greppable.
- **Structs are the ABI interchange currency** — layout-compatible value types
  map to C structs; numeric types map directly. `class`es (which carry
  headers/refcounts) are **not** passed to C directly.
- **`extern`** declarations import C function signatures (manual in v1;
  header-driven binding generation is a later tool, not core).

Refinements crossing C→Clause are discharged by `dynamic` runtime checks
(per ADR-0001). Non-moving deterministic ARC (ADR-0009)
keeps Clause pointers valid for C and gives RAII-style cleanup of C resources via
wrapper-class destructors.

## Why

A safe, verified language needs a hard boundary between its guarantees and
arbitrary C. Isolating raw pointers in their own type and all danger in
`unchecked` regions means the safe subset's proofs are never silently undermined
by FFI.

## Consequences

- Passing a `class` to C requires marshalling through a struct or `Ptr`.
- Manual lifetime care inside `unchecked`; the type system does not track
  `Ptr<T>` validity.
- Auditing FFI safety = reading the `unchecked` regions.
