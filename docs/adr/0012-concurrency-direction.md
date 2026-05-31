# Concurrency: deferred, but committed to message-passing isolation

**Status:** accepted (direction only; full design deferred post-v1)

Full concurrency design is **deferred past v1**, but the **direction is fixed
now**: message-passing / actor-style isolation with **no shared mutable state**.
Threads share *immutable* data freely and exchange *mutable* state only by
passing ownership through channels/messages; no two threads touch the same
mutable object.

## Why

This is the only concurrency model that is race-free **without** a borrow
checker, and it falls out of decisions already made: ADR-0002 bans shared
mutable aliasing, immutability is the default, and value semantics make message
copies natural. A data race is just unsound cross-thread mutable aliasing, which
the safety story already forbids. Shared-memory-plus-locks was rejected because
it needs ownership tracking Clause has deliberately chosen not to build.

## Early-binding consequence

**ARC refcount atomicity** must be anticipated in the object model now: plan for
**non-atomic** refcounts on thread-local objects and **atomic (or a handoff
protocol)** only for immutable data shared across threads. v1 ships
single-threaded with non-atomic refcounts; the object header/runtime is designed
so the distinction can be introduced without a redesign.

## Consequences

- v1 is single-threaded; the door to safe parallelism stays open.
- Concrete model (actors vs CSP channels vs async/await) is chosen later, within
  the no-shared-mutable-state envelope.
