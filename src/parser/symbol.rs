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

    pub fn update_type(&mut self, name: &str, ty: Type) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sym) = scope.symbols.get_mut(name) {
                if sym.mutable {
                    sym.ty = Some(ty);
                    return true;
                }
                return false;
            }
        }
        false
    }

    pub fn snapshot_mut_types(&self) -> HashMap<String, Option<Type>> {
        let mut snap = HashMap::new();
        for scope in self.scopes.iter().rev() {
            for (name, info) in &scope.symbols {
                if info.mutable && !snap.contains_key(name) {
                    snap.insert(name.clone(), info.ty.clone());
                }
            }
        }
        snap
    }

    pub fn unify_mut_types(&mut self, pre_snapshot: &HashMap<String, Option<Type>>) {
        for scope in self.scopes.iter_mut().rev() {
            for (name, info) in scope.symbols.iter_mut() {
                if info.mutable {
                    if let Some(pre_ty) = pre_snapshot.get(name) {
                        if info.ty != *pre_ty {
                            info.ty = Some(Type::Base(crate::parser::ast::BaseType::Any));
                        }
                    }
                }
            }
        }
    }
}
