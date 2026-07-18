# B.U.D. Marketplace

## Scope

This module owns consumer-side B.U.D. data access: `DataAsset`, provenance,
AccessGrant, revocation and marketplace listings. Storage/deal economics remain
in `src/storage/` and are not the same product.

## Current status — blocked P1/P2 repair

The implementation is categorised here but is **not accepted as production
ready**. Before later phases, owner/node signature verification, Address-bound
grants, atomic payment, on-chain ReadOnce consumption, JSON-safe snapshot
encoding, state-root binding, signed actor/RPC flow and CI evidence are required.

See `docs/ARENA2_P1_ACCESSGRANT_DENETIM_2026-07-18.md` and
`docs/ARENA2_P1R0_ACCESSGRANT_TASARIM_2026-07-18.md`.
