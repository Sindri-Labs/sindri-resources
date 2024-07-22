#![allow(dead_code, unused_variables, unused_imports)]

use anyhow::{anyhow, Context as _, Result};

use rand::rngs::OsRng;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;

use num::BigUint;
use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::field::types::PrimeField64;
use plonky2::hash::hash_types::{HashOutTarget, MerkleCapTarget, RichField};
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::iop::target::{BoolTarget, Target};
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{
    CircuitConfig, CommonCircuitData, VerifierCircuitData, VerifierOnlyCircuitData,
};
use plonky2::plonk::config::{AlgebraicHasher, GenericConfig, Hasher, PoseidonGoldilocksConfig};
use plonky2::plonk::proof::{CompressedProofWithPublicInputs, Proof, ProofWithPublicInputs};
use plonky2::plonk::prover::prove;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ProofTuple<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    pub proof: ProofWithPublicInputs<F, C, D>,
    pub vd: VerifierOnlyCircuitData<C, D>,
    pub cd: CommonCircuitData<F, D>,
    pub depth: u32,
}

///Generates a ground proof of the hash chain of batch size B, and exposes its head and tail as public inputs for further composition.
pub fn ground_proof<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const B: usize,
>(
    input: &[F; 4],
    init_value: &[F; 4],
    cutoff: usize,
) -> ProofTuple<F, C, D> {
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    // The arithmetic circuit.
    let targets = hashchain_circuit::<F, C, D, B>(&mut builder);
    let mut pw = PartialWitness::new();
    hashchain_set_partial_witness::<F, C, D, B>(&mut pw, init_value, input, cutoff, &targets);
    builder.print_gate_counts(0);

    let data = builder.build::<C>();
    let proof = data.prove(pw).unwrap();

    ProofTuple {
        proof: proof,
        vd: data.verifier_only,
        cd: data.common,
        depth: 0,
    }
}

pub fn recursive_proof<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    inner_l: &ProofTuple<F, C, D>,
    inner_r: &ProofTuple<F, C, D>,
    min_degree_bits: Option<usize>,
) -> Result<ProofTuple<F, C, D>>
where
    C::Hasher: AlgebraicHasher<F>,
{
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);
    //    assert_eq!(inner_l.depth, inner_r.depth, "Trying to merge proofs of different depth!");

    let pt_l = builder.add_virtual_proof_with_pis(&inner_l.cd);
    let pt_r = builder.add_virtual_proof_with_pis(&inner_r.cd);

    let inner_vdt_l = builder.add_virtual_verifier_data(inner_l.cd.config.fri_config.cap_height);
    let inner_vdt_r = builder.add_virtual_verifier_data(inner_r.cd.config.fri_config.cap_height);

    builder.verify_proof::<C>(&pt_l, &inner_vdt_l, &inner_l.cd);
    builder.verify_proof::<C>(&pt_r, &inner_vdt_r, &inner_r.cd);

    // the output of pt_l is constrained to the input of pt_r
    for i in 0..4 {
        builder.connect(pt_l.public_inputs[i + 4], pt_r.public_inputs[i]);
    }

    for i in 8..12 {
        builder.connect(pt_l.public_inputs[i], pt_r.public_inputs[i]);
    }

    let mut pw = PartialWitness::new();
    pw.set_proof_with_pis_target::<C, D>(&pt_l, &inner_l.proof);
    pw.set_proof_with_pis_target::<C, D>(&pt_r, &inner_r.proof);
    pw.set_verifier_data_target::<C, D>(&inner_vdt_l, &inner_l.vd);
    pw.set_verifier_data_target::<C, D>(&inner_vdt_r, &inner_r.vd);

    for i in 0..4 {
        builder.register_public_input(pt_l.public_inputs[i])
    }
    for i in 4..8 {
        builder.register_public_input(pt_r.public_inputs[i])
    }
    for i in 8..12 {
        builder.connect(pt_l.public_inputs[i], pt_r.public_inputs[i]);
        builder.register_public_input(pt_l.public_inputs[i])
    }

    let data = builder.build::<C>();

    //~~~println!("Meowrging the proofs of depth {} and {}", inner_l.depth, inner_r.depth);
    let proof = data.prove(pw).unwrap();
    Ok(ProofTuple {
        proof: proof,
        vd: data.verifier_only,
        cd: data.common,
        depth: inner_l.depth + 1,
    })
}

pub struct HashchainTargets {
    input: [Target; 4],
    init_value: [Target; 4],
    control_bits: Vec<BoolTarget>,
}

/// Ternary operator for chunks of 4 elements, didn't find any out of the box ¯\_(ツ)_/¯
pub fn ifxab4<F: RichField + Extendable<D>, C: GenericConfig<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    x: &BoolTarget,
    a: &Vec<Target>,
    b: &Vec<Target>,
) -> Vec<Target> {
    //x(a-b)+b
    let mut ans = Vec::new();
    for j in 0..4 {
        let tmp = builder.sub(a[j], b[j]);
        ans.push(builder.arithmetic(F::ONE, F::ONE, x.target, tmp, b[j]))
    }
    ans
}

// pub fn eqab4<
//     F: RichField + Extendable<D>,
//     C:GenericConfig<D>,
//     const D: usize,
//     >(
//     builder: &mut CircuitBuilder<F, D>,
//     a: &Vec<Target>,
//     b: &Vec<Target>
//     )->BoolTarget
// {
//     let tmp = builder.is_equal(a[0], b[0]);
//     for j in 1..4{
//         let tmp = builder.and(tmp, builder.is_equal(a[j], b[j]));
//     }
//     tmp
// }

pub fn hashchain_circuit<
    F: RichField + Extendable<D>,
    C: GenericConfig<D>,
    const D: usize,
    const B: usize,
>(
    builder: &mut CircuitBuilder<F, D>,
) -> HashchainTargets {
    let input: [Target; 4] = builder.add_virtual_targets(4).try_into().unwrap();
    //expose input to the public data (here we do not care about it being public, because it is only used in the inner proofs of the recursive composition)
    builder.register_public_inputs(&input);

    let init_value = builder.add_virtual_targets(4);

    assert!(B > 0, "batch length should be at least 1");

    let mut control_bits = Vec::new();
    for i in 0..B {
        control_bits.push(builder.add_virtual_bool_target_safe())
    }

    let mut run: Vec<Target> = input.to_vec();

    builder.hash_n_to_hash_no_pad::<PoseidonHash>(input.to_vec());

    // in essence, every time x is true, the value is replaced by init_value, and when x = 0 it is Poseidon-ed
    // we do not really care what happens before the last time x is true
    for i in 0..B {
        let tmp = builder
            .hash_n_to_hash_no_pad::<PoseidonHash>(run)
            .elements
            .to_vec();
        run = ifxab4::<F, C, D>(builder, &control_bits[i], &init_value, &tmp);
    }

    let tmp: [Target; 4] = run.try_into().unwrap();
    builder.register_public_inputs(&tmp);

    let init_value_arr: [Target; 4] = init_value.try_into().unwrap(); //I need to convert it into array to register inputs.
    builder.register_public_inputs(&init_value_arr);

    HashchainTargets {
        input: input,
        init_value: init_value_arr,
        control_bits: control_bits,
    }
}

pub fn hashchain_set_partial_witness<
    F: RichField + Extendable<D>,
    C: GenericConfig<D>,
    const D: usize,
    const B: usize,
>(
    pw: &mut PartialWitness<F>,
    init_value: &[F; 4],
    input: &[F; 4],
    cutoff: usize,
    targets: &HashchainTargets,
) {
    pw.set_target_arr(&targets.init_value, init_value);
    assert!(cutoff <= B, "cutoff out of range!"); //cutoff ranges from 0 to B, 0 meaning all Poseidons work and B meaning they are all defunct
    for i in 0..B {
        pw.set_bool_target(targets.control_bits[i], i < cutoff);
    }
    pw.set_target_arr(&targets.input, input);
}

/// This function recursively computes proofs. bin_path variable is the index of a currently processed chunk, which therefore includes batches with indices
/// bin_path*pow(2,depth) to (bin_path+1)*pow(2,depth)-1
pub fn recursive_tree<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    const D: usize,
    const B: usize,
    const DEPTH: usize,
>(
    init_value: [F; 4],
    input: [F; 4],
    cutoff_batch: u128,
    cutoff_step: usize,
    height: usize,
    bin_path: u128,
    trivial_proofs: &Vec<ProofTuple<F, C, D>>,
) -> ProofTuple<F, C, D>
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let depth = DEPTH - height;
    //~~~   println!("Attempting mewge at bin_path={:0b}, height={}", bin_path, height);
    //      let chunk_l = bin_path*u128::pow(2,depth as u32);
    let chunk_r = (bin_path + 1) * u128::pow(2, depth as u32);

    if chunk_r < cutoff_batch {
        // This is the case where the whole chunk consists of trivial proofs
        //~~~       println!("This chunk is fully to the left of the cat-off, retuwning twivial pwoof.");
        trivial_proofs[depth].clone() // This seems weird, I do not actually mutate these proofs anywhere, mb I should fix it at some point.
    } else {
        if depth > 0 {
            // This is the case where the chunk has non-trivial proofs and we need to split it.
            //~~~           println!("Calling a pair of chunks recursively.");
            let tmp = recursive_tree::<F, C, D, B, DEPTH>(
                init_value,
                input,
                cutoff_batch,
                cutoff_step,
                height + 1,
                2 * bin_path,
                trivial_proofs,
            );
            let output: [F; 4] = tmp.proof.public_inputs[4..8].try_into().unwrap();
            let tmp2 = recursive_tree::<F, C, D, B, DEPTH>(
                init_value,
                output,
                cutoff_batch,
                cutoff_step,
                height + 1,
                2 * bin_path + 1,
                trivial_proofs,
            );
            recursive_proof::<F, C, D>(&tmp, &tmp2, None).unwrap()
        } else {
            // Finally, this is the case where we need to calculate ground proof. bin_path == chunk_l == chunk_r is either > or = cutoff_batch.
            //~~~           println!("Non-trivial ground proof. Cat-off status: {}", bin_path==cutoff_batch);
            match bin_path == cutoff_batch {
                true => ground_proof::<F, C, D, B>(&input, &init_value, cutoff_step),
                false => ground_proof::<F, C, D, B>(&input, &init_value, 0),
            }
        }
    }
}

pub fn run<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    global_cutoff: u128,
    init_value: [F; 4],
) -> Result<ProofTuple<F, C, D>>
where
    C::Hasher: AlgebraicHasher<F>,
{
    // This is the amount of stacked hashes we put into the elementary ground proof. In theory, optimal behaviour is having it big enough such that the
    // execution time of the ground proof is ~ equivalent to the recursive proof exec time.

    const BATCH_SIZE: usize = 1024;

    // this is the total depth of recursion batches. pow(2, depth)*BATCH_SIZE hashes must be infeasible to do sequentially.
    const DEPTH: usize = 60;

    // Cutoff is a bit more convenient to actually count from the left, so cutoff_batch and cutoff_step will be counted from the left
    // Global offset from the left equals = pow(2, depth)*BATCH_SIZE - global_cutoff

    let global_cutoff_l: u128 =
        u128::pow(2, DEPTH.try_into()?) * BATCH_SIZE as u128 - global_cutoff;

    let cutoff_batch: u128 = global_cutoff_l / BATCH_SIZE as u128;
    let cutoff_step: usize = global_cutoff_l as usize % BATCH_SIZE;

    let tmp = Instant::now();
    // Trivial proof phase computation, it can be separated into the precompute phase with the small modification of the circuit; and recursive circuit
    // I'm a bit too lazy to mess with this now. This phase will take < time than the main phase in any case.
    let mut trivial_proofs = Vec::new();

    trivial_proofs.push(ground_proof::<F, C, D, BATCH_SIZE>(
        &init_value,
        &init_value,
        BATCH_SIZE,
    ));

    //let mut tmp2: Instant;
    //~~~println!("Gwound trivial proof finishewd. Time spent: {}ms", (tmp2-tmp).as_millis());

    for i in 1..DEPTH {
        trivial_proofs
            .push(recursive_proof(&trivial_proofs[i - 1], &trivial_proofs[i - 1], None).unwrap());
        // tmp2 = Instant::now();
        // println!("Pwocessed depth {}, elapsed time from previous phase {}ms", i, (tmp2-tmp).as_millis());
        // tmp = tmp2;
    }

    // Nontrivial part.

    //~~~println!("Going into recusive tree phase :3. DEPTH={}, cutoff_batch={} which is in binary{:0b}, cutoff_step={}", DEPTH, cutoff_batch, cutoff_batch, cutoff_step);

    let final_proof = recursive_tree::<F, C, D, BATCH_SIZE, DEPTH>(
        init_value,
        init_value,
        cutoff_batch,
        cutoff_step,
        0,
        0,
        &trivial_proofs,
    );

    let tmp2 = Instant::now();
    println!("Computation took {}ms", (tmp2 - tmp).as_millis());
    //tmp = tmp2;

    println!(
        "Allegedly, the result of our poseidon is: {:#x} | {:#x} | {:#x} | {:#x}",
        final_proof.proof.public_inputs[4].to_canonical_u64(),
        final_proof.proof.public_inputs[5].to_canonical_u64(),
        final_proof.proof.public_inputs[6].to_canonical_u64(),
        final_proof.proof.public_inputs[7].to_canonical_u64(),
    );

    let mut tmp = init_value;
    for _ in 0..global_cutoff {
        tmp = PoseidonHash::hash_no_pad(&tmp).elements;
    }
    println!("And the result of naive poseidon of the init_value done {} times is {:#x} | {:#x} | {:#x} | {:#x}", global_cutoff,
    tmp[0].to_canonical_u64(),
    tmp[1].to_canonical_u64(),
    tmp[2].to_canonical_u64(),
    tmp[3].to_canonical_u64()
);

    println!("Proof size: {} bytes\n", final_proof.proof.to_bytes().len());
    Ok(final_proof)
}
