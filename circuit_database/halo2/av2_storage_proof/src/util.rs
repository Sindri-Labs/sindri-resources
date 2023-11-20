use super::Field;
use ethers_core::{
    types::{Address, H256, U256},
    utils::keccak256,
};
use halo2_base::{
    gates::{range::RangeConfig, GateInstructions, RangeInstructions},
    halo2_proofs::circuit::Value,
    utils::{
        bit_length, decompose, decompose_fe_to_u64_limbs, value_to_option, BigPrimeField,
        PrimeField, ScalarField,
    },
    AssignedValue, Context,
    QuantumCell::{Constant, Existing, Witness},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{env::var, fs::File};

pub(crate) const NUM_BYTES_IN_U128: usize = 16;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthConfigParams {
    pub degree: u32,
    // number of SecondPhase advice columns used in RlcConfig
    pub num_rlc_columns: usize,
    // the number of advice columns in phase _ without lookup enabled that RangeConfig uses
    pub num_range_advice: Vec<usize>,
    // the number of advice columns in phase _ with lookup enabled that RangeConfig uses
    pub num_lookup_advice: Vec<usize>,
    pub num_fixed: usize,
    // for keccak chip you should know the number of unusable rows beforehand
    pub unusable_rows: usize,
    pub keccak_rows_per_round: usize,
}

impl EthConfigParams {
    pub fn get_header() -> Self {
        let path =
            var("BLOCK_HEADER_CONFIG").unwrap_or_else(|_| "configs/block_header.json".to_string());
        serde_json::from_reader(
            File::open(&path).unwrap_or_else(|e| panic!("{path} does not exist. {e:?}")),
        )
        .unwrap()
    }
    pub fn get_storage() -> Self {
        let path = var("STORAGE_CONFIG").unwrap_or_else(|_| "configs/storage.json".to_string());
        serde_json::from_reader(
            File::open(&path).unwrap_or_else(|e| panic!("{path} does not exist. {e:?}")),
        )
        .unwrap()
    }
}

pub(crate) type AssignedH256<'v, F> = [AssignedValue<'v, F>; 2]; // H256 as hi-lo (u128, u128)

pub fn get_merkle_mountain_range(leaves: &[H256], max_depth: usize) -> Vec<H256> {
    let num_leaves = leaves.len();
    let mut merkle_roots = Vec::with_capacity(max_depth + 1);
    let mut start_idx = 0;
    for depth in (0..max_depth + 1).rev() {
        if (num_leaves >> depth) & 1 == 1 {
            merkle_roots.push(hash_tree_root(&leaves[start_idx..start_idx + (1 << depth)]));
            start_idx += 1 << depth;
        } else {
            merkle_roots.push(H256::zero());
        }
    }
    merkle_roots
}

pub fn hash_tree_root(leaves: &[H256]) -> H256 {
    let depth = leaves.len().ilog2();
    assert_eq!(leaves.len(), 1 << depth);
    if depth == 0 {
        return leaves[0];
    }
    let mut hash_bytes = leaves
        .chunks(2)
        .map(|pair| keccak256([pair[0].as_bytes(), pair[1].as_bytes()].concat()))
        .collect_vec();
    for d in (0..depth - 1).rev() {
        for i in 0..(1 << d) {
            hash_bytes[i] =
                keccak256([&hash_bytes[2 * i][..], &hash_bytes[2 * i + 1][..]].concat());
        }
    }
    H256::from_slice(&hash_bytes[0])
}

pub fn u256_to_bytes32_be(input: &U256) -> Vec<u8> {
    let mut bytes = vec![0; 32];
    input.to_big_endian(&mut bytes);
    bytes
}

// Field is has PrimeField<Repr = [u8; 32]>
/// Takes hash as bytes32 and returns (hash[..16], hash[16..]) represented as big endian numbers in the prime field
pub fn encode_h256_to_field<F: Field>(hash: &H256) -> [F; 2] {
    let mut bytes = hash.as_bytes().to_vec();
    bytes.reverse();
    // repr is in little endian
    let mut repr = [0u8; 32];
    repr[..16].copy_from_slice(&bytes[16..]);
    let val1 = F::from_repr(repr).unwrap();
    let mut repr = [0u8; 32];
    repr[..16].copy_from_slice(&bytes[..16]);
    let val2 = F::from_repr(repr).unwrap();
    [val1, val2]
}

pub fn decode_field_to_h256<F: Field>(fe: &[F]) -> H256 {
    assert_eq!(fe.len(), 2);
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(&fe[1].to_repr()[..16]);
    bytes[16..].copy_from_slice(&fe[0].to_repr()[..16]);
    bytes.reverse();
    H256(bytes)
}

/// Takes U256, converts to bytes32 (big endian) and returns (hash[..16], hash[16..]) represented as big endian numbers in the prime field
pub fn encode_u256_to_field<F: Field>(input: &U256) -> [F; 2] {
    let mut bytes = vec![0; 32];
    input.to_little_endian(&mut bytes);
    // repr is in little endian
    let mut repr = [0u8; 32];
    repr[..16].copy_from_slice(&bytes[16..]);
    let val1 = F::from_repr(repr).unwrap();
    let mut repr = [0u8; 32];
    repr[..16].copy_from_slice(&bytes[..16]);
    let val2 = F::from_repr(repr).unwrap();
    [val1, val2]
}

pub fn decode_field_to_u256<F: Field>(fe: &[F]) -> U256 {
    assert_eq!(fe.len(), 2);
    let mut bytes = [0u8; 32];
    bytes[16..].copy_from_slice(&fe[0].to_repr()[..16]);
    bytes[..16].copy_from_slice(&fe[1].to_repr()[..16]);
    U256::from_little_endian(&bytes)
}

pub fn encode_addr_to_field<F: Field>(input: &Address) -> F {
    let mut bytes = input.as_bytes().to_vec();
    bytes.reverse();
    let mut repr = [0u8; 32];
    repr[..20].copy_from_slice(&bytes);
    F::from_repr(repr).unwrap()
}

pub fn decode_field_to_addr<F: Field>(fe: &F) -> Address {
    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&fe.to_repr()[..20]);
    bytes.reverse();
    Address::from_slice(&bytes)
}

// circuit utils:

/// Assumes that `bytes` have witnesses that are bytes.
pub fn bytes_be_to_u128<'v, F: PrimeField>(
    ctx: &mut Context<'_, F>,
    gate: &impl GateInstructions<F>,
    bytes: &[AssignedValue<'v, F>],
) -> Vec<AssignedValue<'v, F>> {
    limbs_be_to_u128(ctx, gate, bytes, 8)
}

pub(crate) fn limbs_be_to_u128<'v, F: PrimeField>(
    ctx: &mut Context<'_, F>,
    gate: &impl GateInstructions<F>,
    limbs: &[AssignedValue<'v, F>],
    limb_bits: usize,
) -> Vec<AssignedValue<'v, F>> {
    assert_eq!(128 % limb_bits, 0);
    limbs
        .chunks(128 / limb_bits)
        .map(|chunk| {
            gate.inner_product(
                ctx,
                chunk.iter().rev().map(Existing),
                (0..chunk.len()).map(|idx| Constant(gate.pow_of_two()[limb_bits * idx])),
            )
        })
        .collect_vec()
}

// `num` in u64
pub fn num_to_bytes_be<'v, F: ScalarField>(
    ctx: &mut Context<'v, F>,
    range: &RangeConfig<F>,
    num: &AssignedValue<'v, F>,
    num_bytes: usize,
) -> Vec<AssignedValue<'v, F>> {
    let mut bytes = Vec::with_capacity(num_bytes);
    let pows = range.gate().pow_of_two().iter().step_by(8).take(num_bytes).map(|x| Constant(*x));
    let acc = match value_to_option(num.value()) {
        Some(num) => {
            let byte_vals = decompose_fe_to_u64_limbs(num, num_bytes, 8)
                .into_iter()
                .map(|x| Witness(Value::known(F::from(x))));
            range.gate.inner_product_left(ctx, byte_vals, pows, &mut bytes)
        }
        _ => range.gate.inner_product_left(
            ctx,
            vec![Witness(Value::unknown()); num_bytes],
            pows,
            &mut bytes,
        ),
    };
    ctx.constrain_equal(&acc, num);
    for byte in &bytes {
        range.range_check(ctx, byte, 8);
    }
    bytes.reverse();
    bytes
}

/// Takes a fixed length array `bytes` and returns a length `out_len` array equal to
/// `[[0; out_len - len], bytes[..len]].concat()`, i.e., we take `bytes[..len]` and
/// zero pad it on the left.
///
/// Assumes `0 < len <= max_len <= out_len`.
pub fn bytes_be_var_to_fixed<'v, F: ScalarField>(
    ctx: &mut Context<'_, F>,
    gate: &impl GateInstructions<F>,
    bytes: &[AssignedValue<'v, F>],
    len: &AssignedValue<'v, F>,
    out_len: usize,
) -> Vec<AssignedValue<'v, F>> {
    debug_assert!(bytes.len() <= out_len);
    debug_assert!(bit_length(out_len as u64) < F::CAPACITY as usize);

    // If `bytes` is an RLP field, then `len <= bytes.len()` was already checked during `decompose_rlp_array_phase0` so we don't need to do it again:
    // range.range_check(ctx, len, bit_length(bytes.len() as u64));

    // out[idx] = 1{ len >= out_len - idx } * bytes[idx + len - out_len]
    (0..out_len)
        .map(|idx| {
            let byte_idx = gate.sub(ctx, Existing(len), Constant(F::from((out_len - idx) as u64)));
            // If `len - (out_len - idx) < 0` then the `F` value will be >= `bytes.len()` provided that `out_len` is not too big -- namely `bit_length(out_len) <= F::CAPACITY - 1`
            // Thus select_from_idx at idx < 0 will return 0
            gate.select_from_idx(ctx, bytes.iter().map(|x| Existing(x)), Existing(&byte_idx))
        })
        .collect()
}

pub fn uint_to_bytes_be<'v, F: BigPrimeField>(
    ctx: &mut Context<'v, F>,
    range: &RangeConfig<F>,
    uint: &AssignedValue<'v, F>,
    num_bytes: usize,
) -> Vec<AssignedValue<'v, F>> {
    let mut bytes_le = uint_to_bytes_le(ctx, range, uint, num_bytes);
    bytes_le.reverse();
    bytes_le
}

pub fn uint_to_bytes_le<'v, F: BigPrimeField>(
    ctx: &mut Context<'v, F>,
    range: &RangeConfig<F>,
    uint: &AssignedValue<'v, F>,
    num_bytes: usize,
) -> Vec<AssignedValue<'v, F>> {
    let mut bytes = Vec::with_capacity(num_bytes);
    let pows = range.gate().pow_of_two().iter().step_by(8).take(num_bytes).map(|x| Constant(*x));
    let acc = match value_to_option(uint.value()) {
        Some(uint) => {
            let byte_vals =
                decompose(uint, num_bytes, 8).into_iter().map(|x| Witness(Value::known(x)));
            range.gate.inner_product_left(ctx, byte_vals, pows, &mut bytes)
        }
        _ => range.gate.inner_product_left(
            ctx,
            vec![Witness(Value::unknown()); num_bytes],
            pows,
            &mut bytes,
        ),
    };
    ctx.constrain_equal(&acc, uint);
    for byte in &bytes {
        range.range_check(ctx, byte, 8);
    }
    bytes
}

pub fn bytes_be_to_uint<'v, F: ScalarField>(
    ctx: &mut Context<'_, F>,
    gate: &impl GateInstructions<F>,
    input: &[AssignedValue<'v, F>],
    num_bytes: usize,
) -> AssignedValue<'v, F> {
    gate.inner_product(
        ctx,
        input[..num_bytes].iter().rev().map(Existing),
        (0..num_bytes).map(|idx| Constant(gate.pow_of_two()[8 * idx])),
    )
}
