use super::error::StorageError;
use super::page::{Page, PageId};
use super::storage::Storage;

pub struct BufferPool {
    storage: Storage,
    pages: std::collections::HashMap<PageId, (Page, bool)>, // (page, dirty)
    capacity: usize,
}

//TODO write a better rotation policy
//TODO page allocation
impl BufferPool {
    pub fn new(storage: Storage, capacity: usize) -> Self {
        if capacity == 0 {
            panic!("Buffer pool capacity must not be zero!");
        }
        Self {
            storage,
            pages: std::collections::HashMap::with_capacity(capacity),
            capacity,
        }
    }

    /// Get a page from the cache or load it from storage
    pub fn get_page(&mut self, id: PageId) -> Result<&mut Page, StorageError> {
        if !self.pages.contains_key(&id) {
            let page = self.storage.read_page(id)?;
            // Simple eviction policy: if at capacity, evict a random page
            if self.pages.len() >= self.capacity {
                let mut evict_id = None;
                for (page_id, (_, dirty)) in &self.pages {
                    if !dirty {
                        evict_id = Some(*page_id);
                        break;
                    }
                }

                // If all are dirty, flush one
                if evict_id.is_none() {
                    if let Some((&page_id, _)) = self.pages.iter().next() {
                        let (page, _) = self.pages.remove(&page_id).unwrap();
                        self.storage.write_page(&page)?;
                        evict_id = Some(page_id);
                    }
                }

                //id is always found at this point
                self.pages.remove(&evict_id.unwrap());
            }

            self.pages.insert(id, (page, false));
        }

        let (page, _) = self.pages.get_mut(&id).unwrap();
        Ok(page)
    }

    pub fn mark_dirty(&mut self, id: PageId) -> Result<(), StorageError> {
        if let Some((_, dirty)) = self.pages.get_mut(&id) {
            *dirty = true;
            Ok(())
        } else {
            Err(StorageError::PageNotFound(id))
        }
    }

    pub fn flush(&mut self) -> Result<(), StorageError> {
        for (_, (page, dirty)) in self.pages.iter_mut() {
            if *dirty {
                self.storage.write_page(page)?;
                *dirty = false;
            }
        }

        Ok(())
    }

    pub fn maintenance(&self) -> Result<usize, StorageError> {
        self.storage.maintenance()
    }
}
