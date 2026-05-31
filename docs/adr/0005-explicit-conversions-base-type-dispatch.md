# Explicit conversions only; overloads dispatch on erased base-type signatures

**Status:** accepted

Two linked decisions about resolution and the numeric model:

1. **Overload dispatch ignores refinements.** Constructors (`init`) and any
   overloaded function/method are resolved purely on their **base-type
   signature** — the ordered list of parameter base types, with refinement
   predicates erased. No two overloads may share a base-type signature
   (collisions are rejected at declaration). Parameters may still carry
   refinements as preconditions; they simply do not participate in dispatch.

2. **No implicit conversions.** All type conversions are explicit casts
   (`x as I64`). There is no implicit numeric widening or coercion.

## Why

The constraint solver must never be consulted to pick an overload — resolution
happens early and independently, and a proof obligation that comes back
*Unknown* must not make "which function am I calling?" undecidable. Erasing
refinements before dispatch guarantees this. Forbidding implicit conversions
then makes dispatch **exact-match** (no "most specific overload" tie-breaking),
and independently suits a verified language: implicit coercions silently change
values (narrowing/overflow), whereas an explicit `x as U8` is a visible point
that carries a provable obligation the solver can check.

## Consequences

- `init(x: I32 where x > 0)` and `init(x: I32 where x < 0)` collide on signature
  `(I32)` and are rejected; use distinct base types, distinct arity, or factor
  the precondition differently.
- Every cast is a checkpoint that may carry a range/representability obligation.
- Verbose where C-like code relied on silent promotion — accepted on purpose.
