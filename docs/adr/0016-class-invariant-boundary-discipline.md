# Class-invariant boundary discipline (invariant as implicit pre/post of public methods)

**Status:** accepted

A `class` invariant `I` (the `where` predicate in the class header) is an
**implicit precondition of every public-method *call*** and an **implicit
postcondition of every public-method *body***. These symmetric obligations are
what make "may break `I` temporarily inside a method" sound.

## The rule

- A public method may **assume `I` on entry** and **must re-establish `I` on
  exit** (its implicit postcondition); in between it may break `I`.
- **Every call to a public method must prove `I` at the call site — regardless of
  receiver, `self` included**, and including calls reached through a closure that
  captured `self` or an alias handed to other code.
- **Private helpers carry neither obligation.** Work done while `I` is broken
  happens in private helpers (or inline); you may not call a *public* method until
  `I` is re-established. The same applies during construction: an `init`
  establishes `I` by the time it returns, so any public self-call *before* that
  must already satisfy `I` — otherwise route through private helpers.

## Why

Without the call-site precondition, "break `I` temporarily inside" is **unsound
under reentrancy**: a public method that, while `I` is broken, calls another
public method on `self` would let the callee *assume* a false `I`. Making `I` an
implicit precondition of public-method calls — even self-calls — means a broken
window can never reach a public method, so no proof ever rests on a violated
invariant. Because the obligation lives at the *call*, closure/alias call-backs
into `self` are covered automatically (Refuted ⇒ compile error; Unknown ⇒ runtime
trap — sound either way).

This reuses the existing **modular pre/postcondition** machinery (CONTEXT:
*Modular verification*): `I` is just an implicit conjunct on every public
method's pre and post. It needs **no ghost "validity" field and no pack/unpack
syntax** (contrast Spec#/Boogie), keeping Clause's no-extra-annotation stance.
Forbidding implementation inheritance (ADR-0004) already keeps `I` local and
unweakenable; this ADR adds the temporal discipline that keeps it *true* at every
boundary.

A `struct` invariant needs none of this: value semantics give no aliasing, so it
is checked at each construction/field-write site — no boundaries, no reentrancy.

## Consequences

- **External callers never feel the precondition:** any object obtained from an
  `init` or a returning public method already satisfies `I`, so `I` is available
  for free at external call sites. The precondition only *bites internally* —
  forbidding a public self-call during a broken window.
- You cannot call a public method on a half-updated object; finish the update
  (re-establish `I`) or route the intermediate work through private helpers.
- No reentrancy ghost state; the check is the ordinary VC machinery.
