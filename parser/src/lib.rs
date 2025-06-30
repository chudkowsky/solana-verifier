pub mod annotations;
pub mod builtins;
pub mod json_parser;
pub mod layout;
pub mod stark_proof;
pub mod transform;
pub use stark_proof::*;

pub fn parse<I: AsRef<str>>(input: I) -> anyhow::Result<stark_proof::StarkProof> {
    let proof_json = serde_json::from_str::<json_parser::StarkProof>(input.as_ref())?;
    stark_proof::StarkProof::try_from(proof_json)
}

#[cfg(test)]
mod tests {
    use crate::transform::TransformTo;

    use super::*;

    #[test]
    fn test_parse_recursive_with_poseidon() {
        let input = include_str!("../../example_proof/saya.json");
        let proof_json = serde_json::from_str::<json_parser::StarkProof>(input).unwrap();
        let proof = StarkProof::try_from(proof_json).unwrap();

        let _ = proof.transform_to();
    }
}
