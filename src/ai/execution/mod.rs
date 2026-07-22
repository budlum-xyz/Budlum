//! On-chain AI **execution** primitives (paradigm shift #5 — Agentic Economy).
//!
//! Distinct from attestation (`AiRegistry` k-of-n): execution means a BudZKVM
//! guest ran the model and produced a STARK-checkable binding of
//! (program_hash, input_commitment, output_commitment).
//!
//! v1 ships:
//! - bounded model-class whitelist
//! - fixed-point MLP guest program builder (ISA bytecode)
//! - structural + optional STARK verify hook
//!
//! Production STARK verify of arbitrary guest traces remains mainnet-gated.

mod guest;
mod model_class;
mod verify;

pub use guest::{
    build_fixed_point_mlp_guest, program_hash_from_words, weights_digest, FixedPointMlpSpec,
    MLP_GUEST_VERSION,
};
pub use model_class::{
    AiExecutionModelClass, ModelClassLimits, DEFAULT_EXECUTION_CLASS, MAX_MLP_LAYERS,
    MAX_MLP_PARAMS, MAX_MLP_WIDTH,
};
pub use verify::{verify_execution_proof_structural, ExecutionVerifyReport};
