//! What the view asks of language servers. The app implements `LanguageService` over the servers
//! it runs; an app without any passes `NoLanguageService`, and completion, hover, rename and the
//! other server features then stay off.

use std::sync::Arc;
use std::time::Duration;

use editor_core::{DocumentId, Position};
use lsp_client::client::Callback;
use lsp_client::edits::WorkspaceEdit;
use lsp_client::protocol::{Capabilities, Diagnostic, Range};
use serde_json::Value;

/// Where the answer to a request goes.
pub type Reply = Callback;

/// A request that is waiting for its answer, which can be given up on.
pub struct Pending {
    cancel: Box<dyn Fn() + Send + Sync>,
}

impl Pending {
    /// A pending request that `cancel` gives up on.
    pub fn new(cancel: impl Fn() + Send + Sync + 'static) -> Pending {
        Pending { cancel: Box::new(cancel) }
    }

    /// Tells the server the answer is no longer wanted.
    pub fn cancel(&self) {
        (self.cancel)();
    }
}

/// What applying a workspace edit did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Applied {
    /// How many files were changed, and how many of those were open.
    pub files: usize,
    pub open_files: usize,
    pub edits: usize,
    /// Why some of the edit could not be carried out.
    pub problems: Vec<String>,
}

impl Applied {
    /// A sentence for the status bar.
    pub fn summary(&self) -> String {
        let files = if self.files == 1 { "1 file".to_string() } else { format!("{} files", self.files) };
        let edits = if self.edits == 1 { "1 change".to_string() } else { format!("{} changes", self.edits) };
        let mut sentence = format!("{edits} in {files}");
        if let Some(problem) = self.problems.first() {
            sentence.push_str(&format!(", but {problem}"));
        }
        sentence
    }
}

/// Who formatted, and whether it changed anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formatted {
    pub formatter: &'static str,
    pub changed: bool,
}

/// The language servers behind the documents, as the view uses them.
pub trait LanguageService: Send + Sync {
    /// Whether the user wants inlay hints shown.
    fn inlay_hints_enabled(&self) -> bool;

    /// What the server that has a document open can do; none when no server has it open.
    fn capabilities(&self, document: DocumentId) -> Option<Capabilities>;

    /// The URI the server knows a document by.
    fn uri_of(&self, document: DocumentId) -> Option<String>;

    /// The version of a document the server was last told of.
    fn version_of(&self, document: DocumentId) -> Option<i64>;

    /// What the server last said is wrong in a document, where it touches a range.
    fn diagnostics_at(&self, document: DocumentId, range: &Range) -> Vec<Diagnostic>;

    /// Sends a request about a place in a document, with `extra` added to its params. Returns
    /// none when no server has the document open, and then the reply never runs.
    fn request_at(&self, document: DocumentId, at: Position, method: &str, extra: Value, reply: Reply)
    -> Option<Pending>;

    /// Sends a request that is not about a place, such as resolving a completion item, to the
    /// server of a document.
    fn request_for(&self, document: DocumentId, method: &str, params: Value, reply: Reply) -> Option<Pending>;

    /// Applies a workspace edit to the open documents and the files that are not open.
    fn apply_workspace_edit(&self, edit: &WorkspaceEdit) -> Applied;

    /// Formats a document, waiting at most `limit`. None when no formatter handles it.
    fn format(&self, document: DocumentId, limit: Duration) -> Result<Option<Formatted>, String>;
}

/// The service of an app that runs no language servers.
pub struct NoLanguageService;

impl NoLanguageService {
    /// The service, shared as the view takes it.
    pub fn shared() -> Arc<dyn LanguageService> {
        Arc::new(NoLanguageService)
    }
}

impl LanguageService for NoLanguageService {
    fn inlay_hints_enabled(&self) -> bool {
        false
    }

    fn capabilities(&self, _document: DocumentId) -> Option<Capabilities> {
        None
    }

    fn uri_of(&self, _document: DocumentId) -> Option<String> {
        None
    }

    fn version_of(&self, _document: DocumentId) -> Option<i64> {
        None
    }

    fn diagnostics_at(&self, _document: DocumentId, _range: &Range) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn request_at(
        &self,
        _document: DocumentId,
        _at: Position,
        _method: &str,
        _extra: Value,
        _reply: Reply,
    ) -> Option<Pending> {
        None
    }

    fn request_for(&self, _document: DocumentId, _method: &str, _params: Value, _reply: Reply) -> Option<Pending> {
        None
    }

    fn apply_workspace_edit(&self, _edit: &WorkspaceEdit) -> Applied {
        Applied::default()
    }

    fn format(&self, _document: DocumentId, _limit: Duration) -> Result<Option<Formatted>, String> {
        Ok(None)
    }
}
