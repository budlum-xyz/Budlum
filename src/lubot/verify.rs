//! Faz A — gerçek STARK doğrulama wrapper'ı (bud-proof `DefaultAdapter`).
//!
//! Lubot çıkarım kanıtı, gerçek plonky3 STARK verifier'ı (`bud_proof::DefaultAdapter::verify`)
//! ile doğrulanır. Bu, düny-ilk "doğrulanabilir çıkarım" iddiasının kriptografik
//! çekirdeğidir — mock/hash-commitment değil, gerçek STARK soundness.
//! (v0.2 wrapper seviyesi; gerçek proof üretimi prover tarafında — Faz A derinleşme.)

use bud_proof::{DefaultAdapter, ExecutionPublicInputs, ProofEnvelope, ProverAdapter};

/// Lubot çıkarım kanıtını gerçek plonky3 STARK ile doğrula.
///
/// `proof_bytes` = bincode-serialized `ProofEnvelope`; `expected_inputs` =
/// çıkarımın bağlandığı public input'lar (input/output commit'leri state_root'lara);
/// `program` = çalıştırılan model programı (u64 instruction'ları).
pub fn verify_inference_stark(
    proof_bytes: &[u8],
    expected_inputs: &ExecutionPublicInputs,
    program: &[u64],
) -> Result<(), String> {
    let envelope: ProofEnvelope = bincode::deserialize(proof_bytes)
        .map_err(|e| format!("Lubot STARK: ProofEnvelope deserialize failed: {e}"))?;
    DefaultAdapter::verify(&envelope, expected_inputs, program)
        .map_err(|e| format!("Lubot STARK: verification failed: {e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bud_proof::ExecutionPublicInputs;

    fn inputs() -> ExecutionPublicInputs {
        ExecutionPublicInputs {
            chain_id: 0,
            program_hash: [0; 32],
            initial_state_root: [0; 32],
            final_state_root: [0; 32],
            sender: 0,
            nonce: 0,
            block_height: 0,
            gas_limit: 0,
            gas_used: 0,
            exit_code: 0,
            trace_len: 0,
            event_digest: [0; 32],
        }
    }

    /// Gerçek STARK verifier çağrılır; geçersiz proof reddedilir (InvalidProof/mismatch).
    #[test]
    fn stark_verify_rejects_invalid_proof() {
        // Manuel ProofEnvelope (proof_bytes = çöp) → verify InvalidProof/dahili hata.
        let envelope = ProofEnvelope {
            proof_format_version: 1,
            backend: "plonky3".to_string(),
            p3_version: "0.6".to_string(),
            fri_params_id: "default".to_string(),
            public_inputs_hash: inputs().hash(),
            proof_bytes: vec![0u8; 8],
            degree_bits: 4,
        };
        let bytes = bincode::serialize(&envelope).expect("serialize envelope");
        let res = verify_inference_stark(&bytes, &inputs(), &[]);
        assert!(
            res.is_err(),
            "invalid proof must be rejected by real STARK verifier"
        );
    }

    /// Çöp baytlar deserialize'ta reddedilir.
    #[test]
    fn stark_verify_rejects_garbage_bytes() {
        let res = verify_inference_stark(&[0xFF; 10], &inputs(), &[]);
        assert!(res.is_err(), "garbage bytes must fail");
    }
}
