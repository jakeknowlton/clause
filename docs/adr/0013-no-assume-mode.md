# No `assume` mode: every constraint is proven or runtime-checked

**Status:** accepted

Clause has exactly **two** verification modes — `static` (proven at compile
time) and `dynamic` (runtime-checked) — and deliberately **no `assume`/`admit`
mode** that would inject an unverified fact into the solver. Consequently every
constraint is either *proven* or *runtime-guaranteed*; **nothing is ever merely
trusted.** To keep this sound, the predicate language is restricted to
**runtime-evaluable** forms (bounded quantifiers only).

## Why

A would-be `assume P` requires writing `P` as a boolean expression over in-scope
values and functions — and any such expression can instead be *run* as a
`dynamic` check, which imports the very same fact into the solver's context
*with* the added guarantee that it is actually true (or it traps). So for any
runtime-evaluable predicate, `dynamic` strictly dominates `assume`: same
expressiveness, no soundness hole. The only predicates `assume` could express
that `dynamic` cannot are **non-runtime-evaluable** ones (unbounded quantifiers,
ghost state) — which we exclude from the predicate language. Removing `assume`
therefore loses no expressiveness while closing a soundness hole, which suits
Clause's correctness-first identity.

FFI does not need it: raw-pointer/`extern` danger is contained by the
`unchecked` *region* (ADR-0010), and constraints on C-returned data are enforced
with ordinary `dynamic` runtime checks.

## Consequences

- Refines ADR-0003: `static` predicates may use only `measure` functions;
  `dynamic` predicates any `pure` function — and predicates must be
  runtime-evaluable.
- A fact the solver cannot prove and you cannot afford to runtime-check has no
  escape hatch in v1 (accepted). If non-runtime-evaluable predicates are ever
  admitted, a `trusted`-style mode would have to return — at which point it would
  have a clear, irreplaceable job rather than being a redundant footgun.
