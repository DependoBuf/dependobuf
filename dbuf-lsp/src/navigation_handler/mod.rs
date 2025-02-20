//! Module aims to help with searches in dbuf files.
//!
//! Module should help with such requests:
//! * `textDocument/declaration` (fn goto_declaration)
//! * `textDocument/typeDefinition` (fn goto_type_definition)
//! * `textDocument/references` (fn references)
//! * `textDocument/documentHighlight`
//! * `textDocument/hover`
//!  
//! Also it might be good idea to handle such requests:
//! * `textDocument/prepareTypeHierarchy`
//! * `typeHierarchy/supertypes`
//! * `typeHierarchy/subtypes`
//! * `textDocument/linkedEditingRange`
//!
//! These methods are also about navigation, but there no need to implement them:
//! * `textDocument/definition`
//! * `textDocument/implementation`
//! * `textDocument/prepareCallHierarchy`
//! * `callHierarchy/incomingCalls`
//! * `callHierarchy/outgoingCalls`
//! * `textDocument/documentLink`
//! * `documentLink/resolve`
//!
//!
