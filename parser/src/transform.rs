use crate::stark_proof;
use stark::felt::Felt;
use stark::funvec::FunVec;
use stark::swiftness::air::dynamic::DynamicParams;
use stark::swiftness::air::public_memory::PublicInput as PublicInputVerifier;
use stark::swiftness::air::trace::config::Config as TraceConfigVerifier;
use stark::swiftness::air::trace::Decommitment as TraceDecommitmentVerifier;
use stark::swiftness::air::trace::UnsentCommitment as TraceUnsentCommitmentVerifier;
use stark::swiftness::air::trace::Witness as TraceWitnessVerifier;
use stark::swiftness::air::types::AddrValue;
use stark::swiftness::air::types::Page;
use stark::swiftness::air::types::SegmentInfo as SegmentInfoVerifier;
use stark::swiftness::commitment::table::config::Config as TableConfigVerifier;
use stark::swiftness::commitment::table::types::Decommitment as TableDecommitmentVerifier;
use stark::swiftness::commitment::table::types::Witness as TableCommitmentWitnessVerifier;
use stark::swiftness::commitment::vector::config::Config as VectorConfigVerifier;
use stark::swiftness::commitment::vector::types::Witness as VectorCommitmentWitnessVerifier;
use stark::swiftness::fri::config::Config as FriConfigVerifier;
use stark::swiftness::fri::types::LayerWitness;
use stark::swiftness::fri::types::UnsentCommitment as FriUnsentCommitmentVerifier;
use stark::swiftness::fri::types::Witness as FriWitnessVerifier;
use stark::swiftness::pow::config::Config as PowConfigVerifier;
use stark::swiftness::pow::pow::UnsentCommitment as PowUnsentCommitmentVerifier;
use stark::swiftness::stark::config::StarkConfig as StarkConfigVerifier;
use stark::swiftness::stark::types::StarkProof as StarkProofVerifier;
use stark::swiftness::stark::types::StarkUnsentCommitment as StarkUnsentCommitmentVerifier;
use stark::swiftness::stark::types::StarkWitness as StarkWitnessVerifier;

pub const MONTGOMERY_R: Felt =
    Felt::from_hex_unchecked("0x7FFFFFFFFFFFDF0FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE1");

pub trait TransformTo<T> {
    fn transform_to(self) -> T;
}

impl TransformTo<StarkProofVerifier> for stark_proof::StarkProof {
    fn transform_to(self) -> StarkProofVerifier {
        StarkProofVerifier {
            config: self.config.transform_to(),
            public_input: self.public_input.transform_to(),
            unsent_commitment: self.unsent_commitment.transform_to(),
            witness: self.witness.transform_to(),
        }
    }
}

impl TransformTo<StarkConfigVerifier> for stark_proof::StarkConfig {
    fn transform_to(self) -> StarkConfigVerifier {
        StarkConfigVerifier {
            traces: self.traces.transform_to(),
            composition: self.composition.transform_to(),
            fri: self.fri.transform_to(),
            proof_of_work: self.proof_of_work.transform_to(),
            log_trace_domain_size: self.log_trace_domain_size.into(),
            n_queries: self.n_queries.into(),
            log_n_cosets: self.log_n_cosets.into(),
            n_verifier_friendly_commitment_layers: self
                .n_verifier_friendly_commitment_layers
                .into(),
        }
    }
}

impl TransformTo<PowConfigVerifier> for stark_proof::ProofOfWorkConfig {
    fn transform_to(self) -> PowConfigVerifier {
        PowConfigVerifier {
            n_bits: self.n_bits as u8,
        }
    }
}

impl TransformTo<FriConfigVerifier> for stark_proof::FriConfig {
    fn transform_to(self) -> FriConfigVerifier {
        FriConfigVerifier {
            log_input_size: self.log_input_size.into(),
            n_layers: self.n_layers.into(),
            inner_layers: FunVec::from_vec(
                self.inner_layers
                    .into_iter()
                    .map(|x| x.transform_to())
                    .collect(),
            ),
            fri_step_sizes: FunVec::from_vec(
                self.fri_step_sizes.into_iter().map(|x| x.into()).collect(),
            ),
            log_last_layer_degree_bound: self.log_last_layer_degree_bound.into(),
        }
    }
}

impl TransformTo<TraceConfigVerifier> for stark_proof::TracesConfig {
    fn transform_to(self) -> TraceConfigVerifier {
        TraceConfigVerifier {
            original: self.original.transform_to(),
            interaction: self.interaction.transform_to(),
        }
    }
}

impl TransformTo<TableConfigVerifier> for stark_proof::TableCommitmentConfig {
    fn transform_to(self) -> TableConfigVerifier {
        TableConfigVerifier {
            n_columns: self.n_columns.into(),
            vector: self.vector.transform_to(),
        }
    }
}

impl TransformTo<VectorConfigVerifier> for stark_proof::VectorCommitmentConfig {
    fn transform_to(self) -> VectorConfigVerifier {
        VectorConfigVerifier {
            height: self.height.into(),
            n_verifier_friendly_commitment_layers: self
                .n_verifier_friendly_commitment_layers
                .into(),
        }
    }
}

impl TransformTo<PublicInputVerifier> for stark_proof::PublicInput {
    fn transform_to(self) -> PublicInputVerifier {
        let dynamic_params = match self.dynamic_params.is_empty() {
            true => None,
            false => {
                let params: Vec<u32> = self.dynamic_params.values().cloned().collect();
                Some(DynamicParams::from(params))
            }
        };

        PublicInputVerifier {
            log_n_steps: self.log_n_steps.into(),
            range_check_min: self.range_check_min.into(),
            range_check_max: self.range_check_max.into(),
            layout: self.layout.into(),
            dynamic_params,
            segments: FunVec::from_vec(
                self.segments
                    .into_iter()
                    .map(|x| x.transform_to())
                    .collect(),
            ),
            padding_addr: self.padding_addr.into(),
            padding_value: self.padding_value.into(),
            main_page: Page(FunVec::from_vec(
                self.main_page
                    .into_iter()
                    .map(|x| x.transform_to())
                    .collect(),
            )),
            continuous_page_headers: FunVec::from_vec(vec![]),
        }
    }
}

impl TransformTo<SegmentInfoVerifier> for stark_proof::SegmentInfo {
    fn transform_to(self) -> SegmentInfoVerifier {
        SegmentInfoVerifier {
            begin_addr: self.begin_addr.into(),
            stop_ptr: self.stop_ptr.into(),
        }
    }
}

impl TransformTo<AddrValue> for stark_proof::PubilcMemoryCell {
    fn transform_to(self) -> AddrValue {
        AddrValue {
            address: self.address.into(),
            value: self.value.into(),
        }
    }
}

impl TransformTo<StarkUnsentCommitmentVerifier> for stark_proof::StarkUnsentCommitment {
    fn transform_to(self) -> StarkUnsentCommitmentVerifier {
        StarkUnsentCommitmentVerifier {
            traces: self.traces.transform_to(),
            composition: self.composition.into(),
            oods_values: FunVec::from_vec(self.oods_values.into_iter().map(|x| x.into()).collect()),
            fri: self.fri.transform_to(),
            proof_of_work: self.proof_of_work.transform_to(),
        }
    }
}

impl TransformTo<TraceUnsentCommitmentVerifier> for stark_proof::TracesUnsentCommitment {
    fn transform_to(self) -> TraceUnsentCommitmentVerifier {
        TraceUnsentCommitmentVerifier {
            original: self.original.into(),
            interaction: self.interaction.into(),
        }
    }
}

impl TransformTo<FriUnsentCommitmentVerifier> for stark_proof::FriUnsentCommitment {
    fn transform_to(self) -> FriUnsentCommitmentVerifier {
        FriUnsentCommitmentVerifier {
            last_layer_coefficients: FunVec::from_vec(
                self.last_layer_coefficients
                    .into_iter()
                    .map(|x| x.into())
                    .collect(),
            ),
            inner_layers: FunVec::from_vec(
                self.inner_layers.into_iter().map(|x| x.into()).collect(),
            ),
        }
    }
}

impl TransformTo<PowUnsentCommitmentVerifier> for stark_proof::ProofOfWorkUnsentCommitment {
    fn transform_to(self) -> PowUnsentCommitmentVerifier {
        PowUnsentCommitmentVerifier {
            nonce: self.nonce.to_u64_digits()[0],
        }
    }
}

impl TransformTo<StarkWitnessVerifier> for stark_proof::StarkWitness {
    fn transform_to(self) -> StarkWitnessVerifier {
        StarkWitnessVerifier {
            traces_decommitment: self.traces_decommitment.transform_to(),
            traces_witness: self.traces_witness.transform_to(),
            composition_decommitment: self.composition_decommitment.transform_to(),
            composition_witness: self.composition_witness.transform_to(),
            fri_witness: self.fri_witness.transform_to(),
        }
    }
}

impl TransformTo<TraceDecommitmentVerifier> for stark_proof::TracesDecommitment {
    fn transform_to(self) -> TraceDecommitmentVerifier {
        TraceDecommitmentVerifier {
            original: self.original.transform_to(),
            interaction: self.interaction.transform_to(),
        }
    }
}

impl TransformTo<TableDecommitmentVerifier> for stark_proof::TableDecommitment {
    fn transform_to(self) -> TableDecommitmentVerifier {
        let values = self
            .values
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<_>>();
        TableDecommitmentVerifier {
            montgomery_values: FunVec::from_vec(values.iter().map(|x| x * MONTGOMERY_R).collect()),
            values: FunVec::from_vec(values),
        }
    }
}

impl TransformTo<TraceWitnessVerifier> for stark_proof::TracesWitness {
    fn transform_to(self) -> TraceWitnessVerifier {
        TraceWitnessVerifier {
            original: self.original.transform_to(),
            interaction: self.interaction.transform_to(),
        }
    }
}

impl TransformTo<TableCommitmentWitnessVerifier> for stark_proof::TableCommitmentWitness {
    fn transform_to(self) -> TableCommitmentWitnessVerifier {
        TableCommitmentWitnessVerifier {
            vector: self.vector.transform_to(),
        }
    }
}

impl TransformTo<VectorCommitmentWitnessVerifier> for stark_proof::VectorCommitmentWitness {
    fn transform_to(self) -> VectorCommitmentWitnessVerifier {
        VectorCommitmentWitnessVerifier {
            authentications: FunVec::from_vec(
                self.authentications.into_iter().map(|x| x.into()).collect(),
            ),
        }
    }
}

impl TransformTo<FriWitnessVerifier> for stark_proof::FriWitness {
    fn transform_to(self) -> FriWitnessVerifier {
        FriWitnessVerifier {
            layers: FunVec::from_vec(self.layers.into_iter().map(|x| x.transform_to()).collect()),
        }
    }
}

impl TransformTo<LayerWitness> for stark_proof::FriLayerWitness {
    fn transform_to(self) -> LayerWitness {
        LayerWitness {
            leaves: FunVec::from_vec(self.leaves.into_iter().map(|x| x.into()).collect()),
            table_witness: self.table_witness.transform_to(),
        }
    }
}

impl TransformTo<TableCommitmentWitnessVerifier> for stark_proof::TableCommitmentWitnessFlat {
    fn transform_to(self) -> TableCommitmentWitnessVerifier {
        TableCommitmentWitnessVerifier {
            vector: self.vector.transform_to(),
        }
    }
}

impl TransformTo<VectorCommitmentWitnessVerifier> for stark_proof::VectorCommitmentWitnessFlat {
    fn transform_to(self) -> VectorCommitmentWitnessVerifier {
        VectorCommitmentWitnessVerifier {
            authentications: FunVec::from_vec(
                self.authentications.into_iter().map(|x| x.into()).collect(),
            ),
        }
    }
}
