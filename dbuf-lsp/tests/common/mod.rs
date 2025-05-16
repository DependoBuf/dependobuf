//! Module with helpfull tools for testing.
//!
//! While the parses is not ready contains only:
//! * TEST_URL - url to virtual parsed dbuf file in workspace.
//! * TEST_WORKSPACE - workspace, containing only one file: /src/core/default_ast/sample.dbuf at /testing.dbuf
//!

use std::sync::LazyLock;

use tower_lsp::lsp_types::Url;

use dbuf_lsp::WorkspaceAccess;

pub const TEST_URL: LazyLock<Url> = LazyLock::new(|| Url::from_file_path("/testing.dbuf").unwrap());
pub const TEST_WORKSPACE: LazyLock<WorkspaceAccess> = LazyLock::new(|| {
    let ans = WorkspaceAccess::new();
    ans.open(TEST_URL.clone(), 0, "");
    ans
});
