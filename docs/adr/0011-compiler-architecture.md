# Compiler architecture: staged bootstrap, typed MIR, layered solver

**Status:** accepted

## Bootstrap (staged self-hosting)

- **Stage 0:** a bootstrap compiler written in a host language (TBD — the
  current prototype is in Rust, but the host language is not committed) that
  compiles Clause to LLVM IR.
- **Stage 1:** the compiler is rewritten *in Clause* and compiled by stage 0.
- **Stage 2:** the Clause compiler compiles itself → self-hosted. The bootstrap
  compiler is retained only for cold-bootstrapping new platforms.

## Pipeline

    source → lex → parse → resolve
      → infer/check BASE types (two-layer HM)
      → VERIFY refinements   (modular, on a typed IR, BEFORE monomorphization)
      → monomorphize         (ADR-0007)
      → lower to a typed mid-level IR (SSA/ANF):
            compile pattern matches, insert overflow/constraint runtime checks,
            run ARC + reuse analysis (ADR-0009)
      → emit LLVM IR → LLVM opt → native → link

A **dedicated typed MIR** is where monomorphization, ARC/reuse, runtime-check
insertion, and match-compilation happen — kept off both the AST and raw LLVM IR
(cf. Rust MIR, Swift SIL, Lean IR). Refinement **verification is modular and runs
before monomorphization** (prove each generic function once against its
signature); call sites with concrete values get their obligations checked where
concrete types are known.

## Solver — layered, low-coupled

VCs are discharged behind an abstraction: a **native fast-path discharger**
(constant folding, interval/range reasoning, syntactic implication) handles the
common easy obligations in-compiler, with a **pluggable external SMT backend**
(Z3 initially) for the rest. Z3 is deliberately *not* welded in — the long-term
goal is for more solving (and optimization passes) to be native, leaving
external SMT optional. The solver is a dependency of the *compiler*, never of
compiled programs.

## Consequences

- A REPL is not free under AOT+LLVM; an interpreter/JIT REPL is maintained
  separately and prioritized early.
- The VC abstraction must be designed up front so backends are swappable.
