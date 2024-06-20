use plonky2::{iop::target::Target, plonk::config::{GenericConfig, PoseidonGoldilocksConfig}};
use plonky2::field::types::Field;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};


pub mod circuit;

pub const D: usize = 2;
pub type C = PoseidonGoldilocksConfig;
pub type F = <C as GenericConfig<D>>::F;

