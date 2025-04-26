use bincode::{Decode, Encode};

pub type PageId = u64;

/// Page types supported by the database
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum PageType {
    TableData,
    IndexData,
    Free,
}

/// Header for a database page
#[derive(Debug, Clone, Encode, Decode)]
pub struct PageHeader {
    /// Unique identifier for this page
    pub id: PageId,
    /// Type of page
    pub page_type: PageType,
    pub obj_count: usize,
}

/// A database page that stores data
#[derive(Debug, Clone, Encode, Decode)]
pub struct Page {
    /// Page header
    pub header: PageHeader,
    /// Raw data stored in the page
    pub data: Vec<u8>,
}
