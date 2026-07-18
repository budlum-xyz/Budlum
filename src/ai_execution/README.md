# AI Execution Layer

## Scope

This directory is the dedicated home for **AI Execution Layer** work. It is not
called “AI Inference Layer”: the project covers request execution, verifier
attestations, deterministic outcome recording and later provable execution.

## Current status

- No production AI execution implementation is present here yet.
- `src/registry/role.rs` reserves `AI_VERIFIER` (`RoleId(6)`).
- Phase 10 design and P0 transaction-transport work are prerequisites before
  new execution transactions, RPCs or host-calls are added.
- Until AccessGrant hard enforcement exists, private B.U.D. inputs must not be
  claimed as permission-protected AI inputs.

## Planned boundaries

- execution request/outcome types and canonical state machine;
- verifier Address eligibility (RoleId is eligibility, not identity);
- bounded input/output references and signed attestations;
- separate CI gate and README-reported evidence before mainnet claims.
