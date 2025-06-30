use crate::felt::Felt;
use crate::funvec::{FunVec, FUNVEC_PAGES};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SegmentInfo {
    // Start address of the memory segment.
    pub begin_addr: Felt,
    // Stop pointer of the segment - not necessarily the end of the segment.
    pub stop_ptr: Felt,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Page(pub FunVec<AddrValue, FUNVEC_PAGES>);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct AddrValue {
    pub address: Felt,
    pub value: Felt,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ContinuousPageHeader {
    // Start address.
    pub start_address: Felt,
    // Size of the page.
    pub size: Felt,
    // Hash of the page.
    pub hash: Felt,
    // Cumulative product of the page.
    pub prod: Felt,
}
