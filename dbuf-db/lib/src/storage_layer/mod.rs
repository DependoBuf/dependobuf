pub mod buffer_pool;
pub mod error;
pub mod page;
pub mod paged_storage;
pub mod storage;
pub mod utils;

#[cfg(test)]
mod tests {
    pub mod utility {
        use std::process::Command;

        #[cfg(test)]
        pub fn cleanup(path: &str) {
            Command::new("sh")
                .arg("-c")
                .arg(format!("rm -rf {}", path))
                .output()
                .unwrap();
        }
    }

    use super::*;

    #[test]
    fn storage_test() {
        let path = "temp_path1";
        utility::cleanup(path);

        {
            //create fresh storage
            storage::Storage::new(path, 4096).unwrap();
        }

        {
            let mut storage = storage::Storage::new(path, 2000).unwrap();
            //page size is stored on disk
            assert_eq!(storage.state.page_size, 4096);
            assert_eq!(storage.state.next_page_id, storage::DEFAULT_PAGE);

            let page = storage.allocate_page(page::PageType::Free).unwrap();
            assert_eq!(page.header.id, storage::DEFAULT_PAGE);
            assert_eq!(storage.state.next_page_id, storage::DEFAULT_PAGE + 1);
        }

        {
            let storage = storage::Storage::new(path, 1234).unwrap();
            //next_page_id is stored on disk
            assert_eq!(storage.state.next_page_id, storage::DEFAULT_PAGE + 1);

            let mut page = storage.read_page(storage::DEFAULT_PAGE).unwrap();
            page.data = vec!['a' as u8, 'b' as u8, 'c' as u8];

            storage.write_page(&page).unwrap();
        }

        {
            let mut storage = storage::Storage::new(path, 1337).unwrap();
            assert_eq!(storage.state.next_page_id, storage::DEFAULT_PAGE + 1);

            //page content is stored on disk
            let page = storage.read_page(storage::DEFAULT_PAGE).unwrap();
            assert_eq!(page.data, vec!['a' as u8, 'b' as u8, 'c' as u8]);

            storage.delete_page(page.header.id).unwrap();
        }

        {
            let mut storage = storage::Storage::new(path, 1337).unwrap();
            //page deletion does not mess with next_page_id
            assert_eq!(storage.state.next_page_id, storage::DEFAULT_PAGE + 1);

            assert_eq!(storage.allocate_id().unwrap(), storage::DEFAULT_PAGE);
            assert_eq!(storage.allocate_id().unwrap(), storage::DEFAULT_PAGE + 1);
            assert_eq!(storage.allocate_id().unwrap(), storage::DEFAULT_PAGE + 2);

            storage.free_id(storage::DEFAULT_PAGE + 2).unwrap();
            storage.free_id(storage::DEFAULT_PAGE + 1).unwrap();

            assert_eq!(storage.allocate_id().unwrap(), storage::DEFAULT_PAGE + 2);
            assert_eq!(storage.allocate_id().unwrap(), storage::DEFAULT_PAGE + 1);

            //page is deleted
            let result = storage.read_page(storage::DEFAULT_PAGE);
            assert!(result.is_err());

            //unallocated pages arent found
            let result = storage.read_page(storage::DEFAULT_PAGE + 25);
            assert!(result.is_err());
        }

        utility::cleanup(path);
    }

    #[test]
    fn buffer_pool_test() {
        let path = "temp_path2";
        utility::cleanup(path);

        {
            let mut buffer_pool = buffer_pool::BufferPool::new(path, 4096usize, 3usize).unwrap();

            for i in 0u64..10u64 {
                let page = buffer_pool.allocate_page(page::PageType::Free).unwrap();
                assert_eq!(page.0.header.id, storage::DEFAULT_PAGE + i);
            }
        }

        //allocated pages are stored on disk
        {
            let mut buffer_pool = buffer_pool::BufferPool::new(path, 4096usize, 3usize).unwrap();

            for i in 0u64..10u64 {
                let mut page = buffer_pool.get_page_mut(storage::DEFAULT_PAGE + i).unwrap();
                page.0.data = vec![i as u8; 3];
                page.1 = true;
            }

            buffer_pool.flush().unwrap();
        }

        //changes are stored on disk and readable
        {
            let buffer_pool = buffer_pool::BufferPool::new(path, 4096usize, 3usize).unwrap();

            for i in 0u64..10u64 {
                let page = buffer_pool.get_page(storage::DEFAULT_PAGE + i).unwrap();
                assert_eq!(page.1, false);
                assert_eq!(page.0.data, vec![i as u8; 3]);
            }
        }

        utility::cleanup(path);
    }

    #[test]
    fn paged_storage_test() {
        let path = "temp_path3";
        utility::cleanup(path);

        let page_id: page::PageId;

        {
            let mut paged_storage =
                paged_storage::PagedStorage::new(path, 4096usize, 3usize).unwrap();
            assert_eq!(paged_storage.page_size(), 4096);

            page_id = paged_storage.allocate_page(page::PageType::Free).unwrap();
            assert_eq!(page_id, storage::DEFAULT_PAGE);

            let new_offset = paged_storage.append_data(page_id, &[3u8; 5]).unwrap();
            assert_eq!(new_offset, 5usize);

            paged_storage
                .write_data(page_id, 0usize, &[2u8; 3])
                .unwrap();

            paged_storage.flush().unwrap();
        }

        //written data is saved to disk properly
        {
            let mut paged_storage =
                paged_storage::PagedStorage::new(path, 4096usize, 3usize).unwrap();

            let result = paged_storage.read_data(page_id, 0usize, 5usize).unwrap();
            assert_eq!(result, vec![2u8, 2u8, 2u8, 3u8, 3u8]);

            paged_storage.cut_data(page_id, 2usize).unwrap();
            paged_storage.flush().unwrap();
        }

        //data is cut properly
        {
            let paged_storage = paged_storage::PagedStorage::new(path, 4096usize, 3usize).unwrap();

            let result = paged_storage.read_data(page_id, 0usize, 2usize).unwrap();
            assert_eq!(result, vec![2u8, 2u8]);
        }

        //cant read over bounds
        {
            let mut paged_storage =
                paged_storage::PagedStorage::new(path, 4096usize, 3usize).unwrap();

            let is_invalid = match paged_storage.read_data(page_id, 4095usize, 2usize) {
                Err(error::StorageError::InvalidOperation) => true,
                _ => false,
            };
            assert!(is_invalid);

            //page does not overflow
            let is_overflow = match paged_storage.append_data(page_id, &[5u8; 4095]) {
                Err(error::StorageError::PageFull) => true,
                _ => false,
            };
            assert!(is_overflow);

            let is_overflow = match paged_storage.write_data(page_id, 4095, &[5u8; 2usize]) {
                Err(error::StorageError::PageFull) => true,
                _ => false,
            };
            assert!(is_overflow);
        }

        utility::cleanup(path);
    }
}
