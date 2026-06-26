use crate::parser::ast::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolInfo {
    pub ty: Option<Type>,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
struct Scope {
    symbols: HashMap<String, SymbolInfo>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            symbols: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            scopes: vec![Scope::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare(&mut self, name: String, ty: Option<Type>, mutable: bool) {
        let scope = self.scopes.last_mut().unwrap();
        scope.symbols.insert(name, SymbolInfo { ty, mutable });
    }

    pub fn lookup(&self, name: &str) -> Option<&SymbolInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.symbols.get(name) {
                return Some(sym);
            }
        }
        None
    }

    pub fn exists_in_current(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|s| s.symbols.contains_key(name))
            .unwrap_or(false)
    }
}
