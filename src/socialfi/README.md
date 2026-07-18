# SocialFi

## Scope

SocialFi is a distinct project: native content NFTs, mint/transfer/burn,
creator boosts and the hard-prune link to B.U.D. content. It is separated from
BNS, storage and Marketplace because its ownership and content-lifecycle risk
are different.

## Current code boundary

- `mod.rs` and `types.rs`: SocialFi NFT registry/types.
- Executor, RPC and tests refer to this module through `crate::socialfi`.
- `NftBurn` can trigger the documented B.U.D. hard-pruning flow; this is a
  safety-critical lifecycle boundary, not a Marketplace permission grant.

## Required gate

Existing SocialFi tests remain in the shared Core suite. A separately named
SocialFi CI gate/test inventory must be designed before test counts or
mainnet-readiness claims are made for this project.
