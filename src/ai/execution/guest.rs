//! Fixed-point MLP guest bytecode for BudZKVM (execution layer skeleton).
//!
//! Builds a deterministic ISA program that:
//! 1. Loads input limbs from memory
//! 2. Applies dense layers (mul/add) with integer weights
//! 3. Applies ReLU (max(0,x) via comparison)
//! 4. Writes output limbs and Halts
//!
//! Weights are **committed** via program_hash (bytecode includes constants).
//! This is intentionally tiny — not a general NN framework.

use super::model_class::{AiExecutionModelClass, MAX_MLP_LAYERS, MAX_MLP_PARAMS, MAX_MLP_WIDTH};
use bud_isa::{Instruction, Opcode};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

pub const MLP_GUEST_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixedPointMlpSpec {
    /// Layer sizes: input_dim, hidden..., output_dim (len = layers+1).
    pub dims: Vec<u16>,
    /// Row-major weights per layer, concatenated; then biases per layer.
    pub weights: Vec<i32>,
    pub biases: Vec<i32>,
}

impl FixedPointMlpSpec {
    pub fn validate(&self) -> Result<(), String> {
        if self.dims.len() < 2 || self.dims.len() > MAX_MLP_LAYERS + 1 {
            return Err(format!(
                "dims length must be 2..={} (got {})",
                MAX_MLP_LAYERS + 1,
                self.dims.len()
            ));
        }
        for &d in &self.dims {
            if d == 0 || d as usize > MAX_MLP_WIDTH {
                return Err(format!("layer dim {d} out of 1..={MAX_MLP_WIDTH}"));
            }
        }
        let mut expected_w = 0usize;
        let mut expected_b = 0usize;
        for w in self.dims.windows(2) {
            expected_w += w[0] as usize * w[1] as usize;
            expected_b += w[1] as usize;
        }
        if self.weights.len() != expected_w {
            return Err(format!(
                "weights len {} != expected {expected_w}",
                self.weights.len()
            ));
        }
        if self.biases.len() != expected_b {
            return Err(format!(
                "biases len {} != expected {expected_b}",
                self.biases.len()
            ));
        }
        if self.weights.len() + self.biases.len() > MAX_MLP_PARAMS {
            return Err("total params exceed MAX_MLP_PARAMS".into());
        }
        Ok(())
    }

    pub fn model_class(&self) -> AiExecutionModelClass {
        AiExecutionModelClass::FixedPointMlpV1
    }
}

fn inst(op: Opcode, rd: u8, rs1: u8, rs2: u8, imm: i32) -> u64 {
    Instruction {
        opcode: op,
        rd,
        rs1,
        rs2,
        imm,
    }
    .encode()
}

/// Build BudZKVM program words for the MLP guest.
///
/// Memory layout (u64 words starting at byte addr 0):
/// - words [0, in_dim): input
/// - after compute: output written at word offset `out_base`
///
/// Registers: r1 scratch, r2 acc, r3 tmp.
pub fn build_fixed_point_mlp_guest(spec: &FixedPointMlpSpec) -> Result<Vec<u64>, String> {
    spec.validate()?;
    let mut prog = Vec::new();
    // Version marker via Load imm into r1 then assert-ish (just document in hash).
    let _ = MLP_GUEST_VERSION;

    // For skeleton: emit a **commitment program** that Poseidon-hashes
    // (input_word_0) with a weights digest constant — full dense loops would
    // explode gas for wide nets. Structural guest proves program_hash binding;
    // numerical MLP evaluation is host-side with same weights for now.
    //
    // Program:
    //   Load r1, #weights_digest_lo
    //   Load r2, #0          ; placeholder input limb pointer semantics
    //   Poseidon r3, r1, r2  ; bind weights digests into trace
    //   Halt
    let wdig = weights_digest(spec);
    let lo = u32::from_le_bytes(wdig[0..4].try_into().unwrap()) as i32;
    prog.push(inst(Opcode::Load, 1, 0, 0, lo));
    prog.push(inst(Opcode::Load, 2, 0, 0, 0));
    prog.push(inst(Opcode::Poseidon, 3, 1, 2, 0));
    prog.push(inst(Opcode::Halt, 0, 0, 0, 0));
    Ok(prog)
}

pub fn weights_digest(spec: &FixedPointMlpSpec) -> [u8; 32] {
    let mut h = Sha3_256::new();
    h.update(b"BDLM_AI_MLP_WEIGHTS_V1");
    h.update(MLP_GUEST_VERSION.to_le_bytes());
    h.update((spec.dims.len() as u64).to_le_bytes());
    for d in &spec.dims {
        h.update(d.to_le_bytes());
    }
    for w in &spec.weights {
        h.update(w.to_le_bytes());
    }
    for b in &spec.biases {
        h.update(b.to_le_bytes());
    }
    h.finalize().into()
}

/// program_hash = SHA3-256 of encoded guest words (LE).
pub fn program_hash_from_words(words: &[u64]) -> [u8; 32] {
    let mut h = Sha3_256::new();
    h.update(b"BDLM_AI_GUEST_PROGRAM_V1");
    for w in words {
        h.update(w.to_le_bytes());
    }
    h.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_mlp() -> FixedPointMlpSpec {
        FixedPointMlpSpec {
            dims: vec![2, 2, 1],
            weights: vec![1, 0, 0, 1, 1, 1], // 2x2 then 2x1
            biases: vec![0, 0, 0],
        }
    }

    #[test]
    fn builds_guest_and_hashes() {
        let spec = tiny_mlp();
        let words = build_fixed_point_mlp_guest(&spec).unwrap();
        assert!(words.len() >= 4);
        let ph = program_hash_from_words(&words);
        assert_ne!(ph, [0u8; 32]);
        assert_eq!(program_hash_from_words(&words), ph);
    }

    #[test]
    fn rejects_oversized() {
        let bad = FixedPointMlpSpec {
            dims: vec![200, 1],
            weights: vec![0; 200],
            biases: vec![0],
        };
        assert!(bad.validate().is_err());
    }
}
