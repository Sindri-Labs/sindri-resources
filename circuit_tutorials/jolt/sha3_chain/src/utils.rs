use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

use common::rv_trace::ELFInstruction;
use jolt::{
    field::JoltField,
    jolt::vm::{
        rv32i_vm::{RV32IJoltProof},
        JoltCommitments},
    poly::commitment::commitment_scheme::CommitmentScheme,
};

// This struct is used to store the base64 encoded proof data generated by
// Sindri.
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonProofData {
    pub jolt_proof: String,
    pub jolt_commitments: String,
    pub bytecode: String,
    pub memory_init: String,
}

// This struct stores the JoltProof and JoltCommitments structs.
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct JoltProofStruct<F, PCS>
where
    F: JoltField,
    PCS: CommitmentScheme<Field = F>,
{
    pub proof: RV32IJoltProof<F, PCS>,
    pub commitments: JoltCommitments<PCS>,
}

// This struct stores the bytecode and memory_init data used to recreate the
// preprocessing struct.
#[derive(Serialize, Deserialize)]
pub struct PreprocessingStruct {
    pub bytecode: Vec<ELFInstruction>,
    pub memory_init: Vec<(u64, u8)>,
}

// This function is deserializes the base64 encoded proof data and returns the
// JoltProofStruct and PreprocessingStruct. The bytecode and memory_init fields
// are used to recreate the preprocessing struct which was used to generate the
// proof.  This struct is similar to a verification key in other proving
// frameworks.
pub fn deserialize_jolt_proof_data_from_base64<F, PCS>(
    json_data: JsonProofData,
) -> (JoltProofStruct<F, PCS>, PreprocessingStruct)
where
    F: JoltField,
    PCS: CommitmentScheme<Field = F>,
{
    let jolt_proof_bytes: Vec<u8> = general_purpose::STANDARD
        .decode(json_data.jolt_proof)
        .expect("unable to decode jolt_proof base64 string");
    let jolt_commitments_bytes: Vec<u8> = general_purpose::STANDARD
        .decode(json_data.jolt_commitments)
        .expect("unable to decode jolt_commitments base64 string");
    let bytecode_bytes: Vec<u8> = general_purpose::STANDARD
        .decode(json_data.bytecode)
        .expect("unable to decode jotl bytecodes base64 string");
    let memory_init_bytes: Vec<u8> = general_purpose::STANDARD
        .decode(json_data.memory_init)
        .expect("unable to decode jolt memory_init base64 string");

    let jolt_proof = RV32IJoltProof::<F, PCS>::deserialize_compressed(&*jolt_proof_bytes)
        .expect("unable to deserialize jolt proof");
    let jolt_commitments = JoltCommitments::<PCS>::deserialize_compressed(&*jolt_commitments_bytes)
        .expect("unable to deserialize jolt commitments");

    let jolt_bytecode: Vec<ELFInstruction> =
        rmp_serde::from_slice(&bytecode_bytes).expect("unable to deserialize jolt bytecode");
    let jolt_memory_init: Vec<(u64, u8)> =
        rmp_serde::from_slice(&memory_init_bytes).expect("unable to deserialize jolt memory_init");

    (
        JoltProofStruct { proof: jolt_proof, commitments: jolt_commitments },
        PreprocessingStruct { bytecode: jolt_bytecode, memory_init: jolt_memory_init },
    )
}