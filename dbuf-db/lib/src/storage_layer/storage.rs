use bincode::{Encode, Decode, config};
use marble::{self, Marble};
use std::path::Path;

use super::error::StorageError;
use super::page::{Page, PageHeader, PageId, PageType};

const BINCODE_CONFIG: config::Configuration = config::standard();
const DEFAULT_PAGE : PageId = 100;
const STATE_INDEX : PageId = 0;

#[derive(Encode, Decode)]
struct StorageState {
    pub page_size: usize,
    next_page_id: PageId,
}

pub struct Storage {
    marble: Marble,
    pub state: StorageState,
}

impl Storage {
    /// Create a new storage manager
    pub fn new<P: AsRef<Path>>(path: P, page_size: usize) -> Result<Self, StorageError> {
        let marble = marble::open(path)?;

        let next_page_id = DEFAULT_PAGE;

        Ok(Self {
            marble,
            state: StorageState {
                page_size,
                next_page_id,
            },
        })
    }

    /// Write a page to storage
    pub fn write_page(&self, page: &Page) -> Result<(), StorageError> {
        let encoded: Vec<u8> = bincode::encode_to_vec(page, BINCODE_CONFIG)?;

        self.marble
            .write_batch([(page.header.id, Some(&encoded))])?;

        Ok(())
    }

    /// Allocate a new page of the specified type
    pub fn allocate_page(&mut self, page_type: PageType) -> Result<Page, StorageError> {
        let page_id = self.state.next_page_id;
        self.state.next_page_id += 1;

        let header = PageHeader {
            id: page_id,
            page_type,
            free_space_offset: 0,
        };

        let page = Page {
            header,
            data: Vec::with_capacity(self.state.page_size),
        };

        self.write_page(&page)?;

        Ok(page)
    }

    /// Read a page from storage
    pub fn read_page(&self, id: PageId) -> Result<Page, StorageError> {
        match self.marble.read(id)? {
            Some(data) => {
                let (page, _): (Page, usize) =
                    bincode::decode_from_slice(&data[..], BINCODE_CONFIG)?;
                Ok(page)
            }
            None => Err(StorageError::PageNotFound(id)),
        }
    }

    /// Delete a page from storage
    pub fn delete_page(&self, id: PageId) -> Result<(), StorageError> {
        self.marble
            .write_batch::<&[u8], [(PageId, Option<&[u8]>); 1]>([(id, None)])?;
        Ok(())
    }

    /// Run maintenance to garbage collect and defragment storage
    pub fn maintenance(&self) -> Result<usize, StorageError> {
        let objects_defragmented = self.marble.maintenance()?;
        Ok(objects_defragmented)
    }

    /// Get storage statistics
    pub fn stats(&self) -> marble::Stats {
        self.marble.stats()
    }
}
