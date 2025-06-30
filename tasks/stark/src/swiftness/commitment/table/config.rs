use crate::{felt::Felt, swiftness::commitment::vector};

#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub struct Config {
    pub n_columns: Felt,
    pub vector: vector::config::Config,
}
