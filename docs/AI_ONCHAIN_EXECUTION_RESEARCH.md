# On-Chain AI Execution Layer — Araştırma + İskelet (v1)

**Durum:** İskelet kod main'de (`src/ai/execution/`). Production STARK-full path değil.
**Tarih:** 2026-07-22 · **ARENA2**
**Paradigma:** `docs/03_paradigma_analizi.md` §5 — Agentic Economy / ZK-VM altyapısı.

## Ayrım (kritik)

| Katman | Anlam | Kod |
|---|---|---|
| **Attestation** | k-of-n verifier "çıktı bu" der | `AiRegistry` ✅ |
| **Execution** | Model+input → output BudZKVM STARK bağları | `src/ai/execution` 🔧 iskelet |

## v1 iskelet (ship)

1. **Model class whitelist:** `AiExecutionModelClass::FixedPointMlpV1` + limitler (width/layers/params).
2. **Guest builder:** `build_fixed_point_mlp_guest` → BudZKVM ISA words + `program_hash_from_words`.
3. **Structural verify:** `verify_execution_proof_structural` (commitment/model/proof_bytes).
4. **L1 tx:** `TransactionType::AiAttachExecutionProof` → executor attach + program_hash bind.
5. **AiModelSpec** genişlemesi: `require_execution_proof`, `execution_program_hash`, `execution_class`.

## Bilinçli non-goals (v1)

- Tam dense MLP step-by-step AIR (gas); guest şu an weights-digest Poseidon bağları.
- LLM / float nets.
- MainnetActivation otomatik açma.

## Sonraki

1. Guest'te gerçek matmul loop + prove/verify round-trip (`bud-proof`).
2. `require_execution_proof=true` modellerde result kabulünü proof'a bağla.
3. VerifyInference opcode (0x1F) AIR ↔ `AiExecutionProof.proof_bytes`.

## Paradigma uyumu

Settlement Layer AI ajan ödemelerini (`AiAgentPayment`) zaten taşıyor; execution iskeleti
"ajan çıktısı matematiksel olarak modele bağlı" iddiasının ilk kod köprüsüdür —
attestation'ı kırmadan, opt-in `require_execution_proof` ile.
