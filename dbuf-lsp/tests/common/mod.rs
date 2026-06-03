//! Module with helpfull tools for testing.
//!
//! While the parses is not ready contains only:
//! * `TEST_URL` - url to virtual parsed dbuf file in workspace.
//! * `TEST_WORKSPACE` - workspace, containing only one file: `/src/core/default_ast/sample.dbuf` at `/testing.dbuf`
//!
#![allow(unused, reason = "used in one of tests")]
mod file_generator;

pub use file_generator::FileConfig;
pub use file_generator::FileMetadata;

use std::sync::LazyLock;

use tower_lsp::lsp_types::Url;

pub use dbuf_lsp::WorkspaceAccess;

pub static TEST_URL: LazyLock<Url> =
    LazyLock::new(|| Url::from_file_path("/testing.dbuf").unwrap());

pub static TEST_WORKSPACE: LazyLock<WorkspaceAccess> = LazyLock::new(|| {
    let ans = WorkspaceAccess::new();
    ans.open(TEST_URL.clone(), 0, include_str!("../sample.dbuf"));
    assert!(ans.read(&TEST_URL).get_errors().is_empty());
    ans
});

pub fn empty_workspace() -> WorkspaceAccess {
    WorkspaceAccess::new()
}

pub fn get_url(file: &FileMetadata) -> Url {
    Url::from_file_path(format!("/{}", file.file_name())).expect("valid url")
}

pub fn workspace_with_open_file(workspace: &WorkspaceAccess, file: &FileMetadata) {
    let url = get_url(file);
    let content = file.content();

    workspace.open(url, 0, content);
}

pub fn workspace_with_change_file(workspace: &WorkspaceAccess, file: &FileMetadata) {
    let url = get_url(file);
    let content = file.content();

    let old_version = { workspace.read(&url).get_version() };

    workspace.change(&url, old_version + 1, content);
}

pub fn workspace_with_close_file(workspace: &WorkspaceAccess, file: &FileMetadata) {
    let url = get_url(file);

    workspace.close(&url);
}

#[track_caller]
pub fn check_file(workspace: &WorkspaceAccess, file: &FileMetadata) {
    let url = get_url(file);

    let file_ref = workspace.read(&url);

    let errors = file_ref.get_errors();

    assert!(
        errors.is_empty(),
        "Found some errors while processing file:\n{file:#?}\nErrors:\n{errors:#?}"
    );
}
