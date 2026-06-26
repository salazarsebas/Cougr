use cougr_core::privacy::experimental::CustomCircuit;
use cougr_core::privacy::{Groth16Proof, VerificationKey};
use soroban_sdk::{Bytes, BytesN, Env};

pub(crate) fn build_move_circuit(
    env: &Env,
    vk: &VerificationKey,
    state_hash: &BytesN<32>,
    from: u32,
    to: u32,
) -> CustomCircuit {
    CustomCircuit::builder(vk.clone())
        .add_bytes32(state_hash)
        .add_u32(env, from)
        .add_u32(env, to)
        .build()
}

pub(crate) fn decode_proof(_env: &Env, _proof_bytes: &Bytes) -> Groth16Proof {
    // In a real implementation, this would decode the proof from bytes
    // For now, we create a dummy proof structure
    // The actual proof would be serialized from the off-chain prover
    panic!("Proof decoding not implemented - use mock in tests")
}
