//! Structural verification of AI execution proofs.

use crate::ai::types::{AiExecutionProof, AiInferenceRequest, AiInferenceResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionVerifyReport {
    pub commitments_ok: bool,
    pub model_bound: bool,
    pub has_proof_bytes: bool,
    pub program_hash_nonzero: bool,
}

impl ExecutionVerifyReport {
    pub fn is_structurally_valid(&self) -> bool {
        self.commitments_ok && self.model_bound && self.has_proof_bytes && self.program_hash_nonzero
    }
}

/// Structural checks only (no STARK verify). Used by L1 attach path and tests.
pub fn verify_execution_proof_structural(
    proof: &AiExecutionProof,
    request: &AiInferenceRequest,
    result: &AiInferenceResult,
) -> ExecutionVerifyReport {
    ExecutionVerifyReport {
        commitments_ok: proof.commitments_match(request, result),
        model_bound: proof.model_id == request.model_id,
        has_proof_bytes: !proof.proof_bytes.is_empty(),
        program_hash_nonzero: proof.program_hash != [0u8; 32],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::types::*;
    use crate::core::address::Address;

    #[test]
    fn structural_fail_on_empty_proof() {
        let owner = Address::from([1u8; 32]);
        let mid = AiModelId::of(&owner, &[9u8; 32], 1);
        let req = AiInferenceRequest {
            request_id: AiRequestId([2u8; 32]),
            requester: owner,
            model_id: mid,
            input_commitment: [3u8; 32],
            input_ref: BoundedBytes::empty(),
            max_fee: 0,
            callback: None,
            submitted_at_block: 0,
            deadline_block: 10,
        };
        let res = AiInferenceResult {
            request_id: req.request_id,
            verifier: owner,
            output_commitment: [4u8; 32],
            output_ref: BoundedBytes::empty(),
            result_nonce: 0,
            signature: vec![],
            submitted_at_block: 1,
        };
        let proof = AiExecutionProof {
            model_id: mid,
            input_commitment: req.input_commitment,
            output_commitment: res.output_commitment,
            program_hash: [5u8; 32],
            proof_bytes: vec![],
            steps: 0,
            gas_used: 0,
        };
        let rep = verify_execution_proof_structural(&proof, &req, &res);
        assert!(!rep.is_structurally_valid());
    }
}
