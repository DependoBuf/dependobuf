use super::buffer_pool::BufferPool;

use super::error::StorageError;
use super::page::PageId;
use super::storage::Storage;

use marble::Marble;

use std::path::Path;

pub struct PagedStorage {
    buffer_pool: BufferPool,
}

impl PagedStorage {
    pub fn new<P: AsRef<Path>>(
        path: P,
        page_size: usize,
        buffer_capacity: usize,
    ) -> Result<Self, StorageError> {
        let buffer_pool = BufferPool::new(path, page_size, buffer_capacity)?;
        Ok(Self { buffer_pool })
    }

    pub fn page_size(&self) -> usize {
        self.buffer_pool.page_size()
    }

    pub fn marble(&self) -> &Marble {
        self.buffer_pool.marble()
    }

    pub fn storage(&self) -> &Storage {
        self.buffer_pool.storage()
    }

    pub fn allocate_page(
        &mut self,
        page_type: super::page::PageType,
    ) -> Result<PageId, StorageError> {
        Ok(self.buffer_pool.allocate_page(page_type)?.0.header.id)
    }

    pub fn delete_page(&mut self, id: PageId) -> Result<(), StorageError> {
        self.buffer_pool.delete_page(id)
    }

    /// Write data to a page at the specified offset
    pub fn write_data(
        &mut self,
        page_id: PageId,
        offset: usize,
        data: &[u8],
    ) -> Result<(), StorageError> {
        let page_size = self.page_size();
        let mut page = self.buffer_pool.get_page_mut(page_id)?;

        let data_end = offset + data.len();

        if data_end > page_size {
            return Err(StorageError::PageFull);
        }

        // Ensure the data vector is large enough
        if data_end > page.0.data.len() {
            page.0.data.resize(data_end, 0);
        }

        page.0.data[offset..data_end].copy_from_slice(data);

        page.1 = true;

        Ok(())
    }

    /// Read data from a page
    pub fn read_data(
        &self,
        page_id: PageId,
        offset: usize,
        len: usize,
    ) -> Result<Vec<u8>, StorageError> {
        let page = self.buffer_pool.get_page(page_id)?;

        if offset + len > page.0.data.len() {
            return Err(StorageError::InvalidOperation);
        }

        let result = page.0.data[offset..offset + len].to_vec();

        Ok(result)
    }

    pub fn get_obj_count(&self, page_id: PageId) -> Result<usize, StorageError> {
        let page = self.buffer_pool.get_page(page_id)?;
        Ok(page.0.header.obj_count)
    }

    pub fn set_obj_count(&mut self, page_id: PageId, obj_count: usize) -> Result<(), StorageError> {
        let mut page = self.buffer_pool.get_page_mut(page_id)?;
        page.0.header.obj_count = obj_count;

        Ok(())
    }

    /// Append data to a page
    pub fn append_data(&mut self, page_id: PageId, data: &[u8]) -> Result<usize, StorageError> {
        let page_size = self.page_size();
        let mut page = self.buffer_pool.get_page_mut(page_id)?;

        let offset = page.0.data.len();

        // Ensure the data vector is large enough
        let data_end = offset + data.len();

        if data_end > page_size {
            return Err(StorageError::PageFull);
        }

        page.0.data.resize(data_end, 0);

        // Copy the data
        page.0.data[offset..data_end].copy_from_slice(data);

        page.1 = true;

        Ok(data_end)
    }

    pub fn cut_data(&mut self, page_id: PageId, len: usize) -> Result<(), StorageError> {
        let mut page = self.buffer_pool.get_page_mut(page_id)?;

        let offset = page.0.data.len();

        if len >= offset {
            return Ok(());
        }

        page.0.data.resize(len, 0);
        page.1 = true;

        Ok(())
    }

    /// Flush all dirty pages
    pub fn flush(&mut self) -> Result<(), StorageError> {
        self.buffer_pool.flush()
    }

    /// Run maintenance on the storage
    pub fn maintenance(&self) -> Result<usize, StorageError> {
        self.buffer_pool.maintenance()
    }
}
