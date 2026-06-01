# Clause by Example

A progressive, hand-verified corpus of complete `.cla` programs that exercises
**every keyword, operator, and feature** in the language. Each file is a real,
complete program and is heavily commented.

There is no compiler yet (the restart wiped it), so "exercise" means: every
example is **hand-parsed against [`../docs/grammar.md`](../docs/grammar.md)** and
checked against the ADRs and [`../CONTEXT.md`](../CONTEXT.md). When an example
reveals a gap or contradiction, we fix the grammar/ADR **first**, then write the
example to match. These files are intended to become the parser/checker test
corpus once a front end exists.

## Files

| File | Status | Exercises |
|------|--------|-----------|
| `00-hello.cla` | ✅ | entry point (`main`), `import`, `core`/`std` split, I/O `Result` + `?`, must-use, `return`, `void` |
| `01-bindings.cla` | ✅ | `fix`/`let`, primitive tower, every literal base/suffix, char/string escapes, context-poly literals + I64/F64 defaults, cross-type comparison, constrained mutable binding |
| `02-operators.cla` | ✅ | arithmetic / wrapping / bitwise / shift / comparison / logical / `as` / range / assign-ops, with the ADR-0018 semantics |
| `03-constraints.cla` | ✅ | `where`, set-builder `{v:T\|p}`, named refined types + `it`, constraint-combining, dependent predicates, entailment subtyping, `static`/`dynamic` modes, the trichotomy |
| `04-functions-contracts.cla` | ✅ | pre/postconditions (position rule), inline + relational `where`, expr-body, generics + bounds + turbofish, value-level-over-`T` boundary, `pure` HOF callback |
| `05-measures.cla` | ⬜ | `measure`, `decreases`, structural + well-founded, use in predicates |
| `06-structs.cla` | ⬜ | `struct`, fields, invariants, struct literals |
| `07-classes.cla` | ⬜ | `class`, `init`, methods, invariants, `self`, `static fun`, `weak` |
| `08-enums-matching.cla` | ⬜ | `enum`, variants, `match`, all pattern forms, condition arms, `it`, `Option` |
| `09-interfaces-generics.cla` | ⬜ | `interface`, `impl`, bounds, interface-as-value, `@derive` |
| `10-control-flow.cla` | ⬜ | `if`/`while`/`for` as expressions, `<-`/`break`/`continue`/`return`, loop `else` |
| `11-closures.cla` | ⬜ | lambdas, captures, function types, the purity bit |
| `12-errors.cla` | ⬜ | `Result`, `?`, trap, the two channels |
| `13-collections-strings.cla` | ⬜ | `String`, `Char`, `List`/`Map`/`Set`, `Slice`, `StringBuilder` |
| `14-modules-visibility.cla` | ⬜ | `module`, `import`, `public`/`internal`/private |
| `15-ffi.cla` | ⬜ | `extern`, `unchecked`, `Ptr`, structs as ABI |
| `16-attributes.cla` | ⬜ | `@derive` and other attributes |

## Coverage checklist

Ticked as a real example exercises each item.

**Keywords** — ☑ `fun` `return` `import` `fix` `let` `where` `as` `pure` `type`
`dynamic` `if` `while` `it` `true` `false` `void` + all primitive type names ·
⬜ `measure` `static` `public` `internal` `struct` `class` `enum` `interface`
`impl` `init` `self` `Self` `else` `for` `in` `match` `break` `continue` `module`
`extern` `unchecked` `decreases`

**Operators** — ☑ all: `?` `->` `::` `.` `=>` `<-` turbofish `::<>` · `+ - * / %`
`+% -% *%` `& | ^ ~ << >>` `== != < <= > >=` `&& || !` `= += …` `.. ..=` `as`

**Features** — ☑ entry point · explicit imports / `core`·`std` · I/O
`Result`+`?`+must-use · block→`return` (M2) · primitive tower + all literals ·
context-poly literals + I64/F64 defaults · cross-type comparison · integer
operator semantics (div/mod/shift/wrapping/cast) · constrained mutable binding
(re-checked) · refinements (set-builder, named types, modes, entailment
subtyping, trichotomy) · functions & contracts (pre/post, generics + bounds,
turbofish, purity-bit HOF) · ⬜ measures, struct/class/enum/interface/impl,
pattern matching, closures (full), collections/slices, ARC/`weak`, FFI, attributes
