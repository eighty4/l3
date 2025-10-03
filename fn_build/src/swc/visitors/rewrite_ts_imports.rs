use swc_ecma_ast::{CallExpr, Callee, ImportDecl, Str};
use swc_ecma_visit::{Fold, FoldWith};

pub struct RewriteTsImportsVisitor {
    folding_import_call: bool,
}

impl RewriteTsImportsVisitor {
    pub fn new() -> Self {
        Self {
            folding_import_call: false,
        }
    }
}

impl Fold for RewriteTsImportsVisitor {
    fn fold_call_expr(&mut self, node: CallExpr) -> CallExpr {
        if let Callee::Import(_) = node.callee {
            self.folding_import_call = true;
            node.fold_children_with(self)
        } else {
            node
        }
    }

    fn fold_import_decl(&mut self, mut node: ImportDecl) -> ImportDecl {
        if node.src.value.ends_with(".ts") {
            node.src = Box::from(Str::from(format!(
                "{}{}",
                node.src.value.strip_suffix(".ts").unwrap(),
                ".js"
            )));
        }
        node
    }

    fn fold_str(&mut self, node: Str) -> Str {
        if self.folding_import_call {
            self.folding_import_call = false;
            if node.value.ends_with(".ts") {
                return Str::from(format!(
                    "{}{}",
                    node.value.strip_suffix(".ts").unwrap(),
                    ".js"
                ));
            }
        }
        node
    }
}
