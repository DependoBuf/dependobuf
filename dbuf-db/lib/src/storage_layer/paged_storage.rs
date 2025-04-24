use super::buffer_pool::BufferPool;
use super::error::StorageError;
use super::page::PageId;

use marble::Marble;

pub struct PagedStorage {
    buffer_pool: BufferPool,
}

impl PagedStorage {
    pub fn new(buffer_pool: BufferPool) -> Self {
        Self { buffer_pool }
    }

    pub fn page_size(&self) -> usize {
        self.buffer_pool.page_size()
    }

    pub fn marble(&self) -> &Marble {
        self.buffer_pool.marble()
    }

    pub fn allocate_page(
        &mut self,
        page_type: super::page::PageType,
    ) -> Result<PageId, StorageError> {
        Ok(self.buffer_pool.allocate_page(page_type)?.0.header.id)
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

        if data_end as u32 > page.0.header.free_space_offset {
            page.0.header.free_space_offset = data_end as u32;
        }

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

    /// Append data to a page
    pub fn append_data(&mut self, page_id: PageId, data: &[u8]) -> Result<usize, StorageError> {
        let offset: usize;

        let page_size = self.page_size();
        let mut page = self.buffer_pool.get_page_mut(page_id)?;

        offset = page.0.header.free_space_offset as usize;

        // Ensure the data vector is large enough
        let data_end = offset + data.len();

        if data_end > page_size {
            return Err(StorageError::PageFull);
        }

        if data_end > page.0.data.len() {
            page.0.data.resize(offset + data.len(), 0);
        }

        // Copy the data
        page.0.data[offset..data_end].copy_from_slice(data);

        // Update the free space offset
        page.0.header.free_space_offset = data_end as u32;

        page.1 = true;

        Ok(data_end)
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
