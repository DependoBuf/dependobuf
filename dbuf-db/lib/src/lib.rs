pub mod storage_layer;

mod tests {
    pub mod utility {
        use std::process::Command;

        pub fn cleanup(path: &str) {
            Command::new("sh")
                .arg("-c")
                .arg(format!("rm -rf {}", path))
                .output()
                .unwrap();
        }
    }

    use super::storage_layer::*;

    #[test]
    fn storage_test() {
        let path = "temp_path1";

        {
            let storage = storage::Storage::new(path, 4096).unwrap();
        }

        {
            let mut storage = storage::Storage::new(path, 2000).unwrap();
            //page size is stored on disk
            assert_eq!(storage.state.page_size, 4096);
            assert_eq!(storage.state.next_page_id, storage::DEFAULT_PAGE);

            let page = storage.allocate_page(page::PageType::Free).unwrap();
            assert_eq!(storage.state.next_page_id, storage::DEFAULT_PAGE + 1);
        }

        {
            let mut storage = storage::Storage::new(path, 1234).unwrap();
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

            //page is deleted
            let result = storage.read_page(storage::DEFAULT_PAGE);
            assert!(result.is_err());

            //unallocated pages arent found
            let another_result = storage.read_page(storage::DEFAULT_PAGE + 25);
            assert!(result.is_err());
        }

        utility::cleanup(path);
    }

    #[test]
    fn buffer_pool_test() {}
}
