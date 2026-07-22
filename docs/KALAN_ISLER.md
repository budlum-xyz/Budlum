# Kalan İşler — Budlum

**Güncelleme:** 2026-07-22 (ARENA2) · main

---

## AI execution layer

🔧 **İskelet main'de:** `src/ai/execution/*` + `AiAttachExecutionProof` tx + model class whitelist + MLP guest builder.

⏳ Kalan: full matmul guest prove/verify, VerifyInference AIR, policy enforce on results.

## Intent → zincir (private transfer)

✅ **L1 path:** `L1NoteRegistry` + `PrivateTransferSubmit` / `PrivacyNoteInsert` tx + executor auth (ed25519 over public_digest).

⏳ Relayer mempool UX / fee market tuning; spent_commitment gizleme (TEE proof-only) later.

## Z-B VerifyMerkle 64-depth

✅ KAPANDI (test + CI). MainnetActivation default off.

## BLS/PQ HSM vendor-native

🔧 kısmi kod; ⏳ donanım + audit (Ayaz).

## TEE runtime

🔧 fail-closed stub (wallet); ⏳ SGX/Nitro SDK (donanım).

## Ceremony

⏳ MainnetActivation privacy/merkle flip (Ayaz).

---

*Kod ajanı sahte-yeşil iddia etmez. CI tek hakem.*
