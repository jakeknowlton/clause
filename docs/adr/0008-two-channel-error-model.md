# Two-channel error model: Result values vs non-catchable traps

**Status:** accepted

Clause separates failure into two channels with a sharp dividing line:

- **Recoverable / world-dependent errors** propagate as **`Result<T, E>`
  values** with a `?`-style propagation operator. There are **no exceptions** —
  failure is explicit in the type, control flow stays local, and the solver
  never has to model implicit non-local exits.
- **Proof failures** (a runtime constraint check failing per ADR-0001, an
  overflow trap per ADR-0006, a violated `assume`) **trap/abort, fail-fast, and
  are not catchable** as ordinary errors. At most a top-level last-breath handler
  (log/flush/exit) is permitted; no general recovery.

**The dividing line:** a failure that is *in principle preventable by a proof
obligation on the inputs* is a **constraint** (trap on violation); a failure that
*depends on the external world* is a **`Result`**.

| Operation | Mechanism |
|---|---|
| `xs[i]`, `a / b`, `x as U8` | constraint (provable; trap if unproven+violated) |
| `File.open`, `parse(s)` | `Result<T, E>` (world-dependent) |

## Why

Exceptions are poison for a verifier (every call gains implicit "...or unwinds"
paths). Making proof failures non-catchable keeps the core guarantee crisp ("if
it didn't trap, the constraint held"), avoids mandatory stack unwinding in the
runtime, and eases the GC/FFI story. The proof-vs-world line lets Clause replace
much `Option`/`Result` ceremony (bounds checks, divide-by-zero) with proofs —
a primary ergonomic payoff of the constraint system.

## Consequences

- Partial core operations carry constraints, not `Result`/`Option` wrappers.
- `Result` is reserved for genuinely external failures.
- Traps are unrecoverable; long-running services must avoid reachable traps by
  proving them away or handling the condition before the trapping op.
