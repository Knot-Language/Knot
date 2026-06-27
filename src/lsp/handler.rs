use super::types::*;
use crate::parser::Parser;
use crate::parser::ast::*;
use std::collections::HashMap;

pub struct LspHandler {
    files: HashMap<String, FileState>,
}

impl LspHandler {
    pub fn new() -> Self {
        LspHandler { files: HashMap::new() }
    }

    pub fn handle(&mut self, body: &str) -> Option<JsonRpcResponse> {
        let req: JsonRpcRequest = match serde_json::from_str(body) {
            Ok(r) => r,
            Err(_) => {
                // May be a batch or malformed; skip for now
                return None;
            }
        };

        match req.method.as_str() {
            "initialize" => {
                let id = req.id.clone();
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id,
                    result: Some(serde_json::to_value(InitializeResult {
                        capabilities: ServerCapabilities {
                            text_document_sync: serde_json::json!({
                                "openClose": true,
                                "change": 1  // full sync
                            }),
                            hover_provider: true,
                            definition_provider: true,
                        },
                    }).unwrap()),
                    error: None,
                })
            }
            "initialized" => None, // No response needed
            "shutdown" => {
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: Some(serde_json::Value::Null),
                    error: None,
                })
            }
            "exit" => {
                std::process::exit(0);
            }
            "textDocument/didOpen" => {
                if let Some(params) = req.params {
                    if let Ok(p) = serde_json::from_value::<DidOpenParams>(params) {
                        self.files.insert(
                            p.text_document.uri.clone(),
                            FileState {
                                uri: p.text_document.uri.clone(),
                                text: p.text_document.text.clone(),
                                version: p.text_document.version,
                            },
                        );
                        self.publish_diagnostics(&p.text_document.uri);
                    }
                }
                None
            }
            "textDocument/didChange" => {
                if let Some(params) = req.params {
                    if let Ok(p) = serde_json::from_value::<DidChangeParams>(params) {
                        if let Some(change) = p.content_changes.first() {
                            if let Some(fs) = self.files.get_mut(&p.text_document.uri) {
                                fs.text = change.text.clone();
                                fs.version = p.text_document.version;
                            }
                        }
                        self.publish_diagnostics(&p.text_document.uri);
                    }
                }
                None
            }
            "textDocument/didClose" => {
                if let Some(params) = req.params {
                    if let Ok(p) = serde_json::from_value::<serde_json::Value>(params) {
                        if let Some(uri) = p.get("textDocument").and_then(|d| d.get("uri")).and_then(|u| u.as_str()) {
                            self.files.remove(uri);
                        }
                    }
                }
                None
            }
            "textDocument/hover" => {
                let id = req.id.clone();
                if let Some(params) = req.params {
                    if let Ok(p) = serde_json::from_value::<HoverParams>(params) {
                        let result = self.handle_hover(&p);
                        return Some(JsonRpcResponse {
                            jsonrpc: "2.0".into(),
                            id,
                            result: result.map(|r| serde_json::to_value(r).unwrap()),
                            error: None,
                        });
                    }
                }
                None
            }
            "textDocument/definition" => {
                let id = req.id.clone();
                if let Some(params) = req.params {
                    if let Ok(p) = serde_json::from_value::<DefinitionParams>(params) {
                        let result = self.handle_definition(&p);
                        return Some(JsonRpcResponse {
                            jsonrpc: "2.0".into(),
                            id,
                            result: Some(serde_json::to_value(result).unwrap()),
                            error: None,
                        });
                    }
                }
                None
            }
            _ => {
                // Unknown method — respond with MethodNotFound
                Some(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Method not found: {}", req.method),
                    }),
                })
            }
        }
    }

    fn publish_diagnostics(&self, uri: &str) {
        let file = match self.files.get(uri) {
            Some(f) => f,
            None => return,
        };

        let mut parser = Parser::new(&file.text);
        let _program = parser.parse_program();

        let mut diagnostics = Vec::new();
        for diag in &parser.diagnostics.diagnostics {
            let line = diag.span.line.saturating_sub(1);
            let col = diag.span.col.saturating_sub(1);
            let len = diag.span.len.max(1);
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line: line as u32, character: col as u32 },
                    end: Position { line: line as u32, character: (col + len) as u32 },
                },
                severity: Some(match diag.severity {
                    crate::error::Severity::Error => DiagnosticSeverity::Error,
                    crate::error::Severity::Warning => DiagnosticSeverity::Warning,
                    crate::error::Severity::Note => DiagnosticSeverity::Information,
                }),
                message: diag.message.clone(),
            });
        }

        let notif = JsonRpcNotification {
            jsonrpc: "2.0".into(),
            method: "textDocument/publishDiagnostics".into(),
            params: Some(serde_json::to_value(PublishDiagnosticsParams {
                uri: uri.to_string(),
                diagnostics,
            }).unwrap()),
        };

        let notif_str = serde_json::to_string(&notif).unwrap_or_default();
        let header = format!("Content-Length: {}\r\n\r\n{}", notif_str.len(), notif_str);
        let mut stdout = std::io::stdout();
        let _ = std::io::Write::write_all(&mut stdout, header.as_bytes());
        let _ = std::io::Write::flush(&mut stdout);
    }

    fn handle_hover(&self, params: &HoverParams) -> Option<Hover> {
        let file = self.files.get(&params.text_document.uri)?;
        let source = &file.text;

        let mut parser = Parser::new(source);
        let program = parser.parse_program();

        // Find the token at the given position
        let token_pos = self.find_token_at(source, params.position.line, params.position.character);

        if let Some((token_text, _token_line, _token_col)) = token_pos {
            // Check if it's a function definition
            for stmt in &program {
                if let Stmt::FuncDef { name, generics, params, ret_ty, .. } = stmt {
                    if name == &token_text {
                        let mut info = format!("```knot\nfunc {}", name);
                        if !generics.is_empty() {
                            info.push_str(&format!("[{}]", generics.join(", ")));
                        }
                        info.push('(');
                        let param_strs: Vec<String> = params.iter().map(|p| {
                            let ty_str = p.ty.as_ref().map(|t| format_type(t)).unwrap_or_else(|| "?".into());
                            format!("{}: {}", p.name, ty_str)
                        }).collect();
                        info.push_str(&param_strs.join(", "));
                        info.push(')');
                        if let Some(rt) = ret_ty {
                            info.push_str(&format!(" -> {}", format_type(rt)));
                        }
                        info.push_str("\n```");
                        return Some(Hover {
                            contents: MarkupContent {
                                kind: "markdown".into(),
                                value: info,
                            },
                        });
                    }
                }
            }

            // Check if it's a variable/parameter type
            for stmt in &program {
                if let Stmt::FuncDef { params, .. } = stmt {
                    for p in params {
                        if p.name == token_text {
                            let ty_str = p.ty.as_ref().map(|t| format_type(t)).unwrap_or_else(|| "inferred".into());
                            return Some(Hover {
                                contents: MarkupContent {
                                    kind: "markdown".into(),
                                    value: format!("```knot\n{}: {}\n```", p.name, ty_str),
                                },
                            });
                        }
                    }
                }
            }
        }

        None
    }

    fn handle_definition(&self, params: &DefinitionParams) -> Option<Vec<Location>> {
        let file = self.files.get(&params.text_document.uri)?;
        let source = &file.text;
        let uri = &params.text_document.uri;

        let mut parser = Parser::new(source);
        let program = parser.parse_program();

        let token_pos = self.find_token_at(source, params.position.line, params.position.character);

        if let Some((token_text, _, _)) = token_pos {
            // Search for function definition with this name
            for stmt in &program {
                if let Stmt::FuncDef { name, .. } = stmt {
                    if name == &token_text {
                        // Find the line of the function definition
                        let def_line = self.find_def_line(source, name);
                        return Some(vec![Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position { line: def_line.saturating_sub(1), character: 0 },
                                end: Position { line: def_line.saturating_sub(1), character: name.len() as u32 },
                            },
                        }]);
                    }
                }
            }
        }

        None
    }

    fn find_token_at(&self, source: &str, line: u32, col: u32) -> Option<(String, usize, usize)> {
        let target_line = (line + 1) as usize;
        let target_col = (col + 1) as usize;

        // Tokenize and find the token at the position
        let (tokens, positions) = crate::lexer::lexer::tokenize(source);
        use crate::lexer::token::Token;
        for (i, pos) in positions.iter().enumerate() {
            let (tok_line, tok_col) = pos;
            if *tok_line == target_line && *tok_col <= target_col {
                if i < tokens.len() {
                    if let Token::Identifier(s) = &tokens[i] {
                        if target_col <= tok_col + s.len() {
                            return Some((s.clone(), *tok_line, *tok_col));
                        }
                    }
                }
            }
        }
        None
    }

    fn find_def_line(&self, source: &str, name: &str) -> u32 {
        for (i, line) in source.lines().enumerate() {
            if line.contains(&format!("func {}", name)) || line.contains(&format!("func{}", name)) {
                return (i + 1) as u32;
            }
        }
        1
    }
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Base(b) => match b {
            BaseType::I8 => "I8".into(),
            BaseType::I16 => "I16".into(),
            BaseType::I32 => "I32".into(),
            BaseType::I64 => "I64".into(),
            BaseType::U8 => "U8".into(),
            BaseType::U16 => "U16".into(),
            BaseType::U32 => "U32".into(),
            BaseType::U64 => "U64".into(),
            BaseType::F32 => "F32".into(),
            BaseType::F64 => "F64".into(),
            BaseType::String => "String".into(),
            BaseType::Bool => "Bool".into(),
            BaseType::Null => "Null".into(),
            BaseType::Void => "Void".into(),
            BaseType::Any => "Any".into(),
        },
        Type::Nullable(inner) => format!("{}?", format_type(inner)),
        Type::Named(n) => n.clone(),
        Type::Array(inner) => format!("Array[{}]", format_type(inner)),
    }
}
