#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl Severity {
    pub fn label(&self) -> &str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

impl Span {
    pub fn new(line: usize, col: usize) -> Self {
        Span { line, col, len: 1 }
    }

    pub fn with_len(line: usize, col: usize, len: usize) -> Self {
        Span { line, col, len }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticBag {
    pub diagnostics: Vec<Diagnostic>,
    source: String,
}

impl DiagnosticBag {
    pub fn new(source: &str) -> Self {
        DiagnosticBag {
            diagnostics: Vec::new(),
            source: source.to_string(),
        }
    }

    pub fn push(&mut self, severity: Severity, message: String, span: Span) {
        self.diagnostics.push(Diagnostic {
            severity,
            message,
            span,
            hint: None,
        });
    }

    pub fn push_with_hint(
        &mut self,
        severity: Severity,
        message: String,
        span: Span,
        hint: String,
    ) {
        self.diagnostics.push(Diagnostic {
            severity,
            message,
            span,
            hint: Some(hint),
        });
    }

    pub fn error(&mut self, message: String, span: Span) {
        self.push(Severity::Error, message, span);
    }

    pub fn warn(&mut self, message: String, span: Span) {
        self.push(Severity::Warning, message, span);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn print_all(&self) {
        for diag in &self.diagnostics {
            self.print_diagnostic(diag);
        }
    }

    fn print_diagnostic(&self, diag: &Diagnostic) {
        let label = diag.severity.label();
        let line_no = diag.span.line;
        let col = diag.span.col;

        let source_line = self
            .source
            .lines()
            .nth(line_no.saturating_sub(1))
            .unwrap_or("<unknown>");

        eprintln!(
            "\x1b[1m{}:{}:{}: \x1b[31m{}\x1b[0m\x1b[1m: {}\x1b[0m",
            label, line_no, col, label, diag.message
        );
        eprintln!("  {} | {}", line_no, source_line);

        let padding = line_no.to_string().len() + 3 + (col.saturating_sub(1));
        let underline_len = if diag.span.len > 1 {
            diag.span.len
        } else {
            1usize.max(source_line.len().saturating_sub(col.saturating_sub(1)).min(20))
        };
        let underline = "^".repeat(underline_len);
        eprintln!("  {} \x1b[31m{}\x1b[0m", " ".repeat(padding), underline);

        if let Some(ref hint) = diag.hint {
            eprintln!(
                "  {} \x1b[36m= help: {}\x1b[0m",
                " ".repeat(line_no.to_string().len() + 2),
                hint
            );
        }
    }
}
