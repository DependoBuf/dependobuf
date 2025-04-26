use super::error::StorageError;
use super::page::{Page, PageId, PageType};
use super::storage::Storage;

use marble::Marble;

use std::path::Path;

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

pub struct BufferPool {
    storage: Storage,
    //pages are refcell bc they are a cache that is modified by read ops
    pages: RefCell<HashMap<PageId, (Page, bool)>>, // (page, dirty)
    capacity: usize,
}

//TODO write a better rotation policy
//TODO iterate over all pages in cache
impl BufferPool {
    pub fn new<P: AsRef<Path>>(
        path: P,
        page_size: usize,
        capacity: usize,
    ) -> Result<Self, StorageError> {
        if capacity == 0 {
            panic!("Buffer pool capacity must not be zero!");
        }

        let storage = Storage::new(path, page_size)?;

        Ok(Self {
            storage,
            pages: RefCell::new(HashMap::with_capacity(capacity)),
            capacity,
        })
    }

    pub fn page_size(&self) -> usize {
        self.storage.state.page_size
    }

    pub fn marble(&self) -> &Marble {
        &self.storage.marble
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    fn pop_page(&self) -> Result<(), StorageError> {
        let mut pages = self.pages.borrow_mut();

        let mut evict_id = None;
        for (page_id, (_, dirty)) in pages.iter() {
            if !dirty {
                evict_id = Some(*page_id);
                break;
            }
        }

        // If all are dirty, flush one
        if evict_id.is_none() {
            if let Some((&page_id, _)) = pages.iter().next() {
                let (page, _) = pages.remove(&page_id).unwrap();
                self.storage.write_page(&page)?;
            }
        } else {
            pages.remove(&evict_id.unwrap());
        }

        Ok(())
    }

    pub fn allocate_page<'a>(
        &'a mut self,
        page_type: PageType,
    ) -> Result<RefMut<'a, (Page, bool)>, StorageError> {
        let page = self.storage.allocate_page(page_type)?;

        if self.pages.borrow().len() >= self.capacity {
            self.pop_page()?;
        }

        let mut pages = self.pages.borrow_mut();

        let id = page.header.id;
        pages.insert(id, (page, false));

        Ok(RefMut::map(pages, |p| p.get_mut(&id).unwrap()))
    }

    pub fn delete_page(&mut self, id: PageId) -> Result<(), StorageError> {
        self.pages.borrow_mut().remove(&id);
        self.storage.delete_page(id)?;
        Ok(())
    }

    //place page into cache
    fn bump_page(&self, id: PageId) -> Result<(), StorageError> {
        if !self.pages.borrow().contains_key(&id) {
            let page = self.storage.read_page(id)?;
            // Simple eviction policy: if at capacity, evict a random page
            if self.pages.borrow().len() >= self.capacity {
                self.pop_page()?;
            }

            self.pages.borrow_mut().insert(id, (page, false));
        }
        Ok(())
    }

    /// Get mut ref to page from the cache or load it from storage
    pub fn get_page_mut<'a>(
        &'a mut self,
        id: PageId,
    ) -> Result<RefMut<'a, (Page, bool)>, StorageError> {
        self.bump_page(id)?;

        let pages = self.pages.borrow_mut();

        Ok(RefMut::map(pages, |p| p.get_mut(&id).unwrap()))
    }

    pub fn get_page<'a>(&'a self, id: PageId) -> Result<Ref<'a, (Page, bool)>, StorageError> {
        self.bump_page(id)?;

        let pages = self.pages.borrow();

        Ok(Ref::map(pages, |p| p.get(&id).unwrap()))
    }

    pub fn flush(&mut self) -> Result<(), StorageError> {
        for (_, (page, dirty)) in self.pages.borrow_mut().iter_mut() {
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
