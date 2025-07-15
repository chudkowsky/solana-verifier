use crate::felt::Felt;
use crate::pedersen::constants::{POINTS_P1, POINTS_P2, POINTS_P3, POINTS_P4, SHIFT_POINT};
use lambdaworks_math::elliptic_curve::short_weierstrass::{
    curves::stark_curve::StarkCurve, point::ShortWeierstrassProjectivePoint,
};
use utils::{impl_type_identifiable, BidirectionalStack, Executable, TypeIdentifiable};

pub mod constants;

// Minimal structure - store x and y as they will be overwritten by task results
#[repr(C)]
pub struct PedersenHash {
    phase: u8,
    x_felt: Felt,
    y_felt: Felt,
}

const PHASE_LOOKUP_P1: u8 = 0;
const PHASE_LOOKUP_P2: u8 = 1;
const PHASE_LOOKUP_P3: u8 = 2;
const PHASE_LOOKUP_P4: u8 = 3;
const PHASE_RESULTS: u8 = 4;
const PHASE_FINISHED: u8 = 5;

impl_type_identifiable!(PedersenHash);

impl Default for PedersenHash {
    fn default() -> Self {
        Self::new()
    }
}

impl PedersenHash {
    pub fn new() -> Self {
        Self {
            phase: PHASE_LOOKUP_P1,
            x_felt: Felt::ZERO,
            y_felt: Felt::ZERO,
        }
    }

    pub fn push_input<T: BidirectionalStack>(x: Felt, y: Felt, stack: &mut T) {
        stack.push_front(&x.to_bytes_be()).unwrap();
        stack.push_front(&y.to_bytes_be()).unwrap();
    }
}

impl Executable for PedersenHash {
    fn execute<T: BidirectionalStack>(&mut self, stack: &mut T) -> Vec<Vec<u8>> {
        match self.phase {
            PHASE_LOOKUP_P1 => {
                // Get and store x and y
                self.y_felt = Felt::from_bytes_be_slice(stack.borrow_front());
                stack.pop_front();
                self.x_felt = Felt::from_bytes_be_slice(stack.borrow_front());
                stack.pop_front();

                // Push initial accumulator (SHIFT_POINT)
                stack.push_front(&SHIFT_POINT.x().to_bytes_be()).unwrap();
                stack.push_front(&SHIFT_POINT.y().to_bytes_be()).unwrap();
                stack.push_front(&SHIFT_POINT.z().to_bytes_be()).unwrap();

                self.phase = PHASE_LOOKUP_P2;

                // After executing this task, stack will look like:
                // FRONT: [acc_x_new, acc_y_new, acc_z_new] <- result from LookupAndAccumulate
                let x_bits = self.x_felt.to_bits_le();
                vec![LookupAndAccumulate::new(&x_bits[..248], 1).to_vec_with_type_tag()]
            }
            PHASE_LOOKUP_P2 => {
                // Stack: FRONT: [acc_x, acc_y, acc_z] <- from previous LookupAndAccumulate
                // Accumulator is already on stack, so we don't touch it

                self.phase = PHASE_LOOKUP_P3;
                let x_bits = self.x_felt.to_bits_le();
                vec![LookupAndAccumulate::new(&x_bits[248..252], 2).to_vec_with_type_tag()]
            }
            PHASE_LOOKUP_P3 => {
                // Stack: FRONT: [acc_x, acc_y, acc_z] <- from previous LookupAndAccumulate

                self.phase = PHASE_LOOKUP_P4;
                let y_bits = self.y_felt.to_bits_le();
                vec![LookupAndAccumulate::new(&y_bits[..248], 3).to_vec_with_type_tag()]
            }
            PHASE_LOOKUP_P4 => {
                // Stack: FRONT: [acc_x, acc_y, acc_z] <- from previous LookupAndAccumulate

                self.phase = PHASE_RESULTS;
                let y_bits = self.y_felt.to_bits_le();
                vec![LookupAndAccumulate::new(&y_bits[248..252], 4).to_vec_with_type_tag()]
            }
            PHASE_RESULTS => {
                // Stack: FRONT: [acc_x, acc_y, acc_z] <- final accumulator
                let z = Felt::from_bytes_be_slice(stack.borrow_front());
                stack.pop_front();
                let y = Felt::from_bytes_be_slice(stack.borrow_front());
                stack.pop_front();
                let x = Felt::from_bytes_be_slice(stack.borrow_front());
                stack.pop_front();

                let point =
                    ShortWeierstrassProjectivePoint::<StarkCurve>::new([x.0, y.0, z.0]).unwrap();
                let result = *point.to_affine().x();
                stack.push_front(&result.to_bytes_be()).unwrap();

                self.phase = PHASE_FINISHED;
                vec![]
            }
            PHASE_FINISHED => {
                vec![]
            }
            _ => unreachable!(),
        }
    }

    fn is_finished(&mut self) -> bool {
        self.phase == PHASE_FINISHED
    }
}

// Optimized structure
#[repr(C)]
pub struct LookupAndAccumulate {
    phase: u8,
    table_index: u8,
    chunk_index: u16,
    bits_len: u16,
    // Compact bit representation
    bits_packed: [u32; 8], // 8 * 32 = 256 bits
}

const LA_PHASE_ACCUMULATE: u8 = 0;
const LA_PHASE_FINISHED: u8 = 1;

impl_type_identifiable!(LookupAndAccumulate);

impl LookupAndAccumulate {
    pub fn new(bits: &[bool], table_index: u8) -> Self {
        let mut bits_packed = [0u32; 8];
        let bits_len = bits.len();

        // Pack bits
        for (i, &bit) in bits.iter().enumerate() {
            if bit {
                let word_idx = i / 32;
                let bit_idx = i % 32;
                bits_packed[word_idx] |= 1u32 << bit_idx;
            }
        }

        Self {
            phase: LA_PHASE_ACCUMULATE,
            table_index,
            chunk_index: 0,
            bits_len: bits_len as u16,
            bits_packed,
        }
    }

    #[inline(always)]
    fn get_chunk_offset(&self, chunk_idx: usize) -> usize {
        let start = chunk_idx * PedersenHash::CURVE_CONST_BITS;
        let mut offset = 0;

        for i in 0..PedersenHash::CURVE_CONST_BITS {
            let bit_idx = start + i;
            if bit_idx >= self.bits_len as usize {
                break;
            }

            let word_idx = bit_idx / 32;
            let bit_in_word = bit_idx % 32;

            if (self.bits_packed[word_idx] >> bit_in_word) & 1 == 1 {
                offset |= 1 << i;
            }
        }

        offset
    }
}

impl Executable for LookupAndAccumulate {
    fn execute<T: BidirectionalStack>(&mut self, stack: &mut T) -> Vec<Vec<u8>> {
        match self.phase {
            LA_PHASE_ACCUMULATE => {
                const CHUNK_SIZE: usize = 10;

                let prep: &[ShortWeierstrassProjectivePoint<StarkCurve>] = match self.table_index {
                    1 => &POINTS_P1,
                    2 => &POINTS_P2,
                    3 => &POINTS_P3,
                    4 => &POINTS_P4,
                    _ => unreachable!(),
                };

                // Accumulator is already on stack from previous step
                let z = Felt::from_bytes_be_slice(stack.borrow_front());
                stack.pop_front();
                let y = Felt::from_bytes_be_slice(stack.borrow_front());
                stack.pop_front();
                let x = Felt::from_bytes_be_slice(stack.borrow_front());
                stack.pop_front();

                let mut acc =
                    ShortWeierstrassProjectivePoint::<StarkCurve>::new([x.0, y.0, z.0]).unwrap();

                // Process chunks
                let total_chunks =
                    (self.bits_len as usize).div_ceil(PedersenHash::CURVE_CONST_BITS);
                let start_chunk = self.chunk_index as usize;
                let end_chunk = (start_chunk + CHUNK_SIZE).min(total_chunks);

                for chunk_idx in start_chunk..end_chunk {
                    let offset = self.get_chunk_offset(chunk_idx);
                    if offset > 0 {
                        let table_idx = chunk_idx * PedersenHash::TABLE_SIZE + offset - 1;
                        if table_idx < prep.len() {
                            acc = acc.operate_with_affine(&prep[table_idx]);
                        }
                    }
                }

                self.chunk_index = end_chunk as u16;

                // ALWAYS push accumulator back to front
                stack.push_front(&acc.x().to_bytes_be()).unwrap();
                stack.push_front(&acc.y().to_bytes_be()).unwrap();
                stack.push_front(&acc.z().to_bytes_be()).unwrap();

                if self.chunk_index as usize >= total_chunks {
                    self.phase = LA_PHASE_FINISHED;
                }

                vec![]
            }
            LA_PHASE_FINISHED => {
                vec![]
            }
            _ => unreachable!(),
        }
    }

    fn is_finished(&mut self) -> bool {
        self.phase == LA_PHASE_FINISHED
    }
}
