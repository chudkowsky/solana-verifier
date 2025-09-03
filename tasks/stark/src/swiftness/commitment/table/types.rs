use crate::swiftness::commitment::table::config::{Config, TableConfigBytes};
use crate::swiftness::commitment::vector::config::ConfigTrait;
use crate::swiftness::commitment::vector::types::VectorCommitmentBytes;
use crate::{
    funvec::{FunVec, FUNVEC_DECOMMITMENT_VALUES},
    swiftness::commitment::vector::{self, types::Commitment as VectorCommitment},
};
use felt::Felt;
use utils::BidirectionalStack;

// Commitment for a table (n_rows x n_columns) of field elements in montgomery form.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Commitment {
    pub config: Config,
    pub vector_commitment: vector::types::Commitment,
}

impl Commitment {
    pub fn new(config: Config, vector_commitment: vector::types::Commitment) -> Self {
        Self {
            config,
            vector_commitment,
        }
    }
}

pub trait CommitmentTrait<P, C>: Sized {
    fn from_stack<T: BidirectionalStack>(stack: &mut T) -> Self;
    fn push_to_stack<T: BidirectionalStack>(&self, stack: &mut T);
    fn to_bytes_be(&self) -> (P, C);
}

impl CommitmentTrait<VectorCommitmentBytes, TableConfigBytes> for Commitment {
    /// Read Query from stack: index first, then value
    fn from_stack<T: BidirectionalStack>(stack: &mut T) -> Self {
        let config = Config::from_stack(stack);
        let value = VectorCommitment::from_stack(stack);
        Self::new(config, value)
    }

    /// Push Query to stack: value first, then index (reverse order for stack)
    fn push_to_stack<T: BidirectionalStack>(&self, stack: &mut T) {
        self.vector_commitment.push_to_stack(stack);
        self.config.push_to_stack(stack);
    }

    fn to_bytes_be(&self) -> (VectorCommitmentBytes, TableConfigBytes) {
        (
            self.vector_commitment.to_bytes_be(),
            self.config.to_bytes_be(),
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Decommitment {
    pub values: FunVec<Felt, FUNVEC_DECOMMITMENT_VALUES>,
    pub montgomery_values: FunVec<Felt, FUNVEC_DECOMMITMENT_VALUES>,
}

impl CommitmentTrait<Decommitment, ()> for Decommitment {
    fn from_stack<T: BidirectionalStack + StarkVerifyTrait>(stack: &mut T) {
        // Read values length
        let values_len = Felt::from_bytes_be_slice(stack.borrow_front());
        stack.pop_front();
        let count = values_len.to_biguint().try_into().unwrap();
        println!("count: {:?}", count);

        // Read decommitment_values
        for i in 0..count {
            let value = Felt::from_bytes_be_slice(stack.borrow_front());
            stack.pop_front();
            let verify_variables: &mut VerifyVariables = stack.get_verify_variables_mut();
            verify_variables.decommitment_values[i] = value;
        }

        // Read montgomery_values
        for i in 0..count {
            let value = Felt::from_bytes_be_slice(stack.borrow_front());
            stack.pop_front();
            let verify_variables: &mut VerifyVariables = stack.get_verify_variables_mut();
            verify_variables.montgomery_values[i] = value;
        }
    }

    fn push_to_stack<T: BidirectionalStack + StarkVerifyTrait>(&mut self, stack: &mut T) {
        // Get count first
        let count = Felt::from_bytes_be_slice(stack.borrow_front());
        stack.pop_front();
        let count_usize: usize = count.to_biguint().try_into().unwrap();

        // Push montgomery_values in reverse order - no allocation
        for i in (0..count_usize).rev() {
            let value_bytes = {
                let verify_variables: &mut VerifyVariables = stack.get_verify_variables_mut();
                verify_variables.montgomery_values[i].to_bytes_be()
            };
            stack.push_front(&value_bytes).unwrap();
        }
        stack
            .push_front(&Felt::from(count_usize).to_bytes_be())
            .unwrap();

        // Push decommitment_values in reverse order - no allocation
        for i in (0..count_usize).rev() {
            let value_bytes = {
                let verify_variables: &mut VerifyVariables = stack.get_verify_variables_mut();
                verify_variables.decommitment_values[i].to_bytes_be()
            };
            stack.push_front(&value_bytes).unwrap();
        }
        stack
            .push_front(&Felt::from(count_usize).to_bytes_be())
            .unwrap();
    }

    fn from_stack_ref<T: BidirectionalStack + StarkVerifyTrait>(_stack: &T) -> &Self {
        // For Decommitment, data is stored in VerifyVariables, use from_stack instead
        unimplemented!("Decommitment data is stored in VerifyVariables, use from_stack instead")
    }

    fn to_bytes_be(&self) -> Decommitment {
        *self
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Witness {
    pub vector: vector::types::Witness,
}

impl CommitmentTrait<Witness, ()> for Witness {
    fn from_stack<T: BidirectionalStack + StarkVerifyTrait>(stack: &mut T) {
        vector::types::Witness::from_stack(stack);
    }

    fn from_stack_ref<T: BidirectionalStack + StarkVerifyTrait>(_stack: &T) -> &Self {
        // For Witness, data is stored in VerifyVariables, use from_stack instead
        unimplemented!("Witness data is stored in VerifyVariables, use from_stack instead")
    }

    fn push_to_stack<T: BidirectionalStack + StarkVerifyTrait>(&mut self, stack: &mut T) {
        self.vector.push_to_stack(stack);
    }

    fn to_bytes_be(&self) -> Witness {
        *self
    }
}
