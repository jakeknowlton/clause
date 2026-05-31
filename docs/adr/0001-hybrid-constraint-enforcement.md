# Hybrid constraint enforcement with visible runtime fallback

**Status:** accepted

Clause constraints (refinement predicates) are enforced by a hybrid strategy
driven by an SMT solver, which yields a **trichotomy** per constraint:

- **Proven** (negation unsatisfiable) — holds on every reachable path. No
  runtime check emitted.
- **Refuted** (a concrete, *reachable* counterexample exists) — definitely
  violated. **Compile error.**
- **Unknown** (timeout / undecidable / insufficient facts) — insert a **runtime
  check** that traps on violation.

Crucially, the runtime fallback (Unknown only) is **not silent** — it produces a
compiler diagnostic (warning, or an explicit opt-in), so the developer always
knows where the prover gave up and what it cost.

A counterexample only counts as **Refuted** when it is genuinely reachable —
i.e. produced from the precise decidable core. Once over-approximating features
(measures, nonlinear arithmetic) are involved, a counterexample may be
**spurious** and must be treated as **Unknown**, never **Refuted**, so that
correct programs are never rejected. Rule: error only on a concrete
counterexample from the decidable core; everything fuzzier degrades to Unknown.

## Considered options

- **Sound-by-rejection (Dafny/F\*):** unproven ⇒ compile error. Rejected: too
  much friction for a general-purpose language, and undecidable predicates can
  wedge otherwise-correct programs.
- **Silent hybrid:** insert runtime checks with no diagnostic. Rejected: the
  developer can't tell which constraints survived to runtime, hiding both
  correctness risk and performance cost.
- **Visible hybrid (chosen):** always enforce, fall back to runtime checks, but
  surface the fallback.

## Consequences

- Clause guarantees every constraint holds (statically *or* via a runtime trap),
  so there are no "unchecked" constraints.
- Requires both an SMT solver in the toolchain *and* a runtime trap mechanism
  (ties into the panic/abort model — TBD).
- The compiler must report per-constraint proof status; this becomes a
  user-facing feature, not just an internal detail.
