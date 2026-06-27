use serde::{Deserialize, Serialize};

// ── JSON-RPC ────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

// ── Common LSP types ────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<DiagnosticSeverity>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

// ── Initialize ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct InitializeParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
}

#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    pub text_document_sync: serde_json::Value,
    pub hover_provider: bool,
    pub definition_provider: bool,
}

// ── Text Document ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DidOpenParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Deserialize)]
pub struct DidChangeParams {
    #[serde(rename = "textDocument")]
    pub text_document: VersionedTextDocumentIdentifier,
    #[serde(rename = "contentChanges")]
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

#[derive(Debug, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    #[serde(rename = "languageId")]
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

#[derive(Debug, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    pub text: String,
}

// ── Hover ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct HoverParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Debug, Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

#[derive(Debug, Serialize)]
pub struct Hover {
    pub contents: MarkupContent,
}

#[derive(Debug, Serialize)]
pub struct MarkupContent {
    pub kind: String,
    pub value: String,
}

// ── Definition ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DefinitionParams {
    #[serde(rename = "textDocument")]
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

// ── Publish Diagnostics ─────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}

// ── Known files ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FileState {
    pub uri: String,
    pub text: String,
    pub version: i32,
}

impl Default for FileState {
    fn default() -> Self {
        FileState {
            uri: String::new(),
            text: String::new(),
            version: 0,
        }
    }
}
