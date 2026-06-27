use std::io::{self, Read, Write};
use crate::error::DiagnosticBag;
use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;

pub fn run() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Read Content-Length header
        let mut header = String::new();
        loop {
            let mut byte = [0u8; 1];
            if stdin.read_exact(&mut byte).is_err() {
                return;
            }
            header.push(byte[0] as char);
            if header.ends_with("\r\n\r\n") {
                break;
            }
        }

        // Parse Content-Length
        let len: usize = header
            .lines()
            .find(|l| l.starts_with("Content-Length: "))
            .and_then(|l| l["Content-Length: ".len()..].trim().parse().ok())
            .unwrap_or(0);

        if len == 0 {
            continue;
        }

        // Read body
        let mut buf = vec![0u8; len];
        if stdin.read_exact(&mut buf).is_err() {
            return;
        }
        let body = String::from_utf8_lossy(&buf).to_string();

        let response = handle_message(&body);
        if let Some(resp) = response {
            stdout.write_all(resp.as_bytes()).ok();
            stdout.flush().ok();
        }
    }
}

fn handle_message(body: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    let method = json.get("method")?.as_str()?;
    let id = json.get("id").cloned();
    let params = json.get("params");

    match method {
        "initialize" => {
            let result = serde_json::json!({
                "capabilities": {
                    "textDocumentSync": { "openClose": true, "change": 1 },
                    "hoverProvider": true,
                    "completionProvider": { "triggerCharacters": ["."] }
                },
                "serverInfo": { "name": "knot-lsp", "version": "0.1.0" }
            });
            Some(make_response(id, &result))
        }
        "initialized" => None,
        "shutdown" => {
            let resp = make_response(id, &serde_json::json!(null));
            // Send exit notification after shutdown response
            // (handled by client; we just return the response)
            Some(resp)
        }
        "exit" => {
            std::process::exit(0);
        }
        "textDocument/didOpen" => {
            let uri = params?.get("textDocument")?.get("uri")?.as_str()?;
            let text = params?.get("textDocument")?.get("text")?.as_str()?;
            let diags = check_document(text);
            send_diagnostics(uri, &diags)
        }
        "textDocument/didChange" => {
            let uri = params?.get("textDocument")?.get("uri")?.as_str()?;
            let changes = params?.get("contentChanges")?.as_array()?;
            if let Some(change) = changes.last() {
                if let Some(text) = change.get("text")?.as_str() {
                    let diags = check_document(text);
                    return send_diagnostics(uri, &diags);
                }
            }
            None
        }
        "textDocument/hover" => {
            let hover = serde_json::json!({
                "contents": { "kind": "plaintext", "value": "Knot" }
            });
            Some(make_response(id, &hover))
        }
        "textDocument/completion" => {
            let result = serde_json::json!({
                "isIncomplete": false,
                "items": [
                    {"label": "func", "kind": 14, "detail": "function definition"},
                    {"label": "class", "kind": 7, "detail": "class definition"},
                    {"label": "enum", "kind": 13, "detail": "enum definition"},
                    {"label": "if", "kind": 14, "detail": "if statement"},
                    {"label": "else", "kind": 14, "detail": "else clause"},
                    {"label": "while", "kind": 14, "detail": "while loop"},
                    {"label": "for", "kind": 14, "detail": "for-in loop"},
                    {"label": "return", "kind": 14, "detail": "return statement"},
                    {"label": "match", "kind": 14, "detail": "match expression"},
                    {"label": "throw", "kind": 14, "detail": "throw exception"},
                    {"label": "try", "kind": 14, "detail": "try-catch block"},
                    {"label": "import", "kind": 14, "detail": "import module"},
                    {"label": "static", "kind": 14, "detail": "static method"},
                    {"label": "private", "kind": 14, "detail": "private member"},
                    {"label": "abstract", "kind": 14, "detail": "abstract class"},
                    {"label": "mixin", "kind": 14, "detail": "mixin inclusion"},
                    {"label": "operator", "kind": 14, "detail": "operator overloading"},
                    {"label": "wrap", "kind": 14, "detail": "wrap decorator"},
                    {"label": "new", "kind": 14, "detail": "constructor"},
                    {"label": "assert", "kind": 14, "detail": "assertion"},
                    {"label": "true", "kind": 14, "detail": "boolean true"},
                    {"label": "false", "kind": 14, "detail": "boolean false"},
                    {"label": "null", "kind": 14, "detail": "null value"},
                    {"label": "Any", "kind": 14, "detail": "dynamic type"},
                    {"label": "I32", "kind": 6, "detail": "32-bit signed integer"},
                    {"label": "I64", "kind": 6, "detail": "64-bit signed integer"},
                    {"label": "F64", "kind": 6, "detail": "64-bit float"},
                    {"label": "String", "kind": 6, "detail": "string type"},
                    {"label": "Bool", "kind": 6, "detail": "boolean type"},
                    {"label": "Void", "kind": 6, "detail": "void type"}
                ]
            });
            Some(make_response(id, &result))
        }
        _ => None,
    }
}

fn check_document(text: &str) -> DiagnosticBag {
    let mut parser = Parser::new(text);
    let program = parser.parse_program();

    let mut diagnostics = DiagnosticBag::new(text);
    diagnostics.diagnostics.append(&mut parser.diagnostics.diagnostics);

    match SemanticAnalyzer::analyze(&program, &mut diagnostics) {
        Err(errors) => {
            for e in errors {
                let span = find_error_span(text, &e);
                diagnostics.push(crate::error::Severity::Error, e, span);
            }
        }
        Ok(_) => {}
    }

    diagnostics
}

fn find_error_span(source: &str, msg: &str) -> crate::error::Span {
    // Extract variable/type name from error messages like:
    // "type mismatch: cannot assign Base(String) to b: Base(I32)"
    // "undefined variable: x"
    // Look for identifier-like tokens after "to ", "variable: ", etc.
    for pattern in &["to ", "variable: ", "function '", "of ", "for "] {
        if let Some(pos) = msg.find(pattern) {
            let after = &msg[pos + pattern.len()..];
            let name: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if !name.is_empty() {
                return find_ident_span(source, &name);
            }
        }
    }
    // Fallback: search for any word-like token in the last part of the message
    crate::error::Span::new(1, 1)
}

fn find_ident_span(source: &str, name: &str) -> crate::error::Span {
    for (line_no, line) in source.lines().enumerate() {
        if let Some(col) = line.find(name) {
            // Verify it's a whole word (surrounded by non-alphanumeric chars)
            let before_ok = col == 0 || !line.as_bytes()[col - 1].is_ascii_alphanumeric() && line.as_bytes()[col - 1] != b'_';
            let after = col + name.len();
            let after_ok = after >= line.len() || !line.as_bytes()[after].is_ascii_alphanumeric() && line.as_bytes()[after] != b'_';
            if before_ok && after_ok {
                return crate::error::Span::with_len(line_no + 1, col + 1, name.len());
            }
        }
    }
    crate::error::Span::new(1, 1)
}

fn send_diagnostics(uri: &str, diags: &DiagnosticBag) -> Option<String> {
    let mut lsp_diags = Vec::new();
    for d in &diags.diagnostics {
        let severity = match d.severity {
            crate::error::Severity::Error => 1,
            crate::error::Severity::Warning => 2,
            crate::error::Severity::Note => 3,
        };
        let line = d.span.line.saturating_sub(1);
        let col = d.span.col.saturating_sub(1);
        lsp_diags.push(serde_json::json!({
            "range": {
                "start": { "line": line, "character": col },
                "end": { "line": line, "character": col + d.span.len.max(1) }
            },
            "severity": severity,
            "message": d.message,
            "source": "knot"
        }));
    }

    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": lsp_diags
        }
    });

    let body = serde_json::to_string(&notification).unwrap_or_default();
    Some(format!("Content-Length: {}\r\n\r\n{}", body.len(), body))
}

fn make_response(id: Option<serde_json::Value>, result: &serde_json::Value) -> String {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    });
    let body = serde_json::to_string(&response).unwrap_or_default();
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
}
