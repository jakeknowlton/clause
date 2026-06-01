# Package architecture: `core` / `std` split with an auto-imported `core::prelude`

**Status:** accepted

Clause's library is split into two packages, and exactly one prelude is implicit.

- **`core`** — the intrinsic layer shipped with the compiler (CONTEXT's "tiny
  intrinsic core"): primitives, `[T, N]`, `Option`, `Result`, `String`,
  `Ptr`/FFI, the growable collections (`List`/`Map`/`Set`), and the fundamental
  interfaces (`Eq`/`Ord`/`Hash`/`Show`/`Iterator`/`Add`/…). The freestanding line
  is **"no OS / no syscalls"**, *not* "no allocator": ARC allocation is Clause's
  universal memory model, so `String` and the collections (all ARC-managed) live
  in `core`.
- **`std`** — the hosted standard library written in Clause: `std::io`
  (`print`/`println`/`File`/`Writer`/`Reader`), `std::time`, `std::env`,
  `std::process`, … everything that needs an operating system.

## Prelude policy: only `core::prelude` is implicit

- **`core::prelude`** is auto-imported into every module: the core types
  (`Option`, `Result`, `String`, `List`, `Ptr`) and the fundamental interfaces —
  the set the grammar already calls "prelude types." It is inert *vocabulary*
  (types + interfaces), never behavior.
- **`std` is always explicit.** There is no implicit `std::prelude`; I/O and
  everything hosted require an `import` (`import std::io::{println};`). A reader
  can always see *where a name comes from* from the import list.

## Why

- The `core`/`std` split yields a **freestanding/embedded target for free** — a
  systems language with FFI will eventually want `core`-only, no-OS builds; the
  line is drawn now, cheaply, matching the layering CONTEXT already describes.
- **Explicit `std` imports over an implicit `std::prelude`**: provenance is
  visible at the top of every file and nothing effectful is "magically in scope."
  Effect *safety* is already handled finer-grained by the purity system (a
  non-`pure` function is greppable in its own signature), so an implicit prelude
  would trade the language's prized explicitness (explicit conversions, no glob
  imports, greppable `unchecked`) for mere convenience.

## Consequences

- Every program that prints imports `std::io` — `00-hello.cla` included. Mild
  friction, accepted for provenance clarity.
- `core` assumes the ARC runtime/allocator; a true no-allocator target is a
  non-goal (contrast Rust's `core`/`alloc` split — Clause's allocator is
  universal).
- The precise membership of `core::prelude` beyond the grammar's named prelude
  types (e.g. whether `Map`/`Set` are auto-imported or sit behind an explicit
  `core::collections` import) is refinable as the collection surface settles.
