use crate::{
    felt::Felt,
    funvec::{FunVec, FUNVEC_SEGMENTS},
};

use super::{
    dynamic::DynamicParams,
    types::{ContinuousPageHeader, Page, SegmentInfo},
};

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PublicInput {
    pub log_n_steps: Felt,
    pub range_check_min: Felt,
    pub range_check_max: Felt,
    pub layout: Felt,
    pub dynamic_params: Option<DynamicParams>,
    pub segments: FunVec<SegmentInfo, FUNVEC_SEGMENTS>,
    pub padding_addr: Felt,
    pub padding_value: Felt,
    pub main_page: Page,
    pub continuous_page_headers: FunVec<ContinuousPageHeader, 0>,
}
