use bincode::{Decode, Encode, config};
use marble::Marble;
use std::path::Path;

use super::error::StorageError;
use super::page::{Page, PageHeader, PageId, PageType};

use super::utils::{BINCODE_CONFIG, load, save};

pub const DEFAULT_PAGE: PageId = 100;
const STATE_INDEX: PageId = 0;

#[derive(Encode, Decode)]
pub struct StorageState {
    pub page_size: usize,
    pub next_page_id: PageId,
}

pub struct Storage {
    pub marble: Marble,
    pub state: StorageState,
}

//TODO refactor encoding and decoding into generic fn
impl Storage {
    /// Create a new storage manager
    /// page_size is ignored if state was already written down
    pub fn new<P: AsRef<Path>>(path: P, page_size: usize) -> Result<Self, StorageError> {
        let marble = marble::open(path)?;

        if let Some(state) = load(&marble, STATE_INDEX)? {
            return Ok(Self { marble, state });
        }

        let state = StorageState {
            page_size,
            next_page_id: DEFAULT_PAGE,
        };

        save(&marble, &state, STATE_INDEX)?;

        Ok(Self { marble, state })
    }

    /// Write a page to storage
    pub fn write_page(&self, page: &Page) -> Result<(), StorageError> {
        save(&self.marble, page, page.header.id)
    }

    /// Allocate a new page of the specified type
    pub fn allocate_page(&mut self, page_type: PageType) -> Result<Page, StorageError> {
        let page_id = self.state.next_page_id;
        self.state.next_page_id += 1;

        save(&self.marble, &self.state, STATE_INDEX)?;

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
        match load::<Page>(&self.marble, id)? {
            Some(page) => Ok(page),
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
}
