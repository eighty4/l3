use swc_ecma_ast::{CallExpr, Callee, ImportDecl, Str};
use swc_ecma_visit::{Fold, FoldWith};

pub struct CollectImportsVisitor {
    folding_import_call: bool,
    imports: Vec<String>,
}

impl CollectImportsVisitor {
    pub fn new() -> Self {
        Self {
            folding_import_call: false,
            imports: Vec::new(),
        }
    }

    pub fn result(self) -> Vec<String> {
        self.imports
    }
}

impl Fold for CollectImportsVisitor {
    fn fold_call_expr(&mut self, node: CallExpr) -> CallExpr {
        if let Callee::Import(_) = node.callee {
            self.folding_import_call = true;
            node.fold_children_with(self)
        } else {
            node
        }
    }

    fn fold_import_decl(&mut self, node: ImportDecl) -> ImportDecl {
        self.imports.push(node.src.value.to_string());
        node
    }

    fn fold_str(&mut self, node: Str) -> Str {
        if self.folding_import_call {
            self.imports.push(node.value.to_string());
            self.folding_import_call = false
        }
        node
    }
}
