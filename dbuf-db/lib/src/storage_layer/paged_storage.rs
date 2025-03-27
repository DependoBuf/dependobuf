use super::buffer_pool::BufferPool;
use super::error::StorageError;
use super::page::PageId;
use super::storage::Storage;

pub struct PagedStorage {
    buffer_pool: BufferPool,
}

impl PagedStorage {
    pub fn new(storage: Storage, buffer_capacity: usize) -> Self {
        let buffer_pool = BufferPool::new(storage, buffer_capacity);

        Self { buffer_pool }
    }

    /// Write data to a page at the specified offset
    pub fn write_data(
        &mut self,
        page_id: PageId,
        offset: usize,
        data: &[u8],
    ) -> Result<(), StorageError> {
        let page = self.buffer_pool.get_page(page_id)?;

        let data_end = offset + data.len();

        // Ensure the data vector is large enough
        if data_end > page.data.len() {
            page.data.resize(data_end, 0);
        }

        page.data[offset..data_end].copy_from_slice(data);

        let end_offset = offset + data.len();
        if end_offset as u32 > page.header.free_space_offset {
            page.header.free_space_offset = end_offset as u32;
        }

        self.buffer_pool.mark_dirty(page_id)?;

        Ok(())
    }

    /// Read data from a page
    pub fn read_data(
        &mut self,
        page_id: PageId,
        offset: usize,
        len: usize,
    ) -> Result<Vec<u8>, StorageError> {
        let page = self.buffer_pool.get_page(page_id)?;

        if offset + len > page.data.len() {
            return Err(StorageError::InvalidOperation);
        }

        let result = page.data[offset..offset + len].to_vec();

        Ok(result)
    }

    /// Append data to a page
    pub fn append_data(&mut self, page_id: PageId, data: &[u8]) -> Result<usize, StorageError> {
        let page = self.buffer_pool.get_page(page_id)?;

        let offset = page.header.free_space_offset as usize;

        // Ensure the data vector is large enough
        if offset + data.len() > page.data.capacity() {
            return Err(StorageError::PageFull);
        }

        if offset + data.len() > page.data.len() {
            page.data.resize(offset + data.len(), 0);
        }

        // Copy the data
        page.data[offset..offset + data.len()].copy_from_slice(data);

        // Update the free space offset
        page.header.free_space_offset = (offset + data.len()) as u32;

        self.buffer_pool.mark_dirty(page_id)?;

        Ok(offset)
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
