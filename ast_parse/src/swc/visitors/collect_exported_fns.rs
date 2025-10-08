use swc_ecma_ast::{Decl, ExportDecl, ModuleDecl};
use swc_ecma_visit::Fold;

#[derive(Default)]
pub struct CollectExportedFnsVisitor {
    exported_fns: Vec<String>,
}

impl CollectExportedFnsVisitor {
    pub fn new() -> Self {
        Self {
            exported_fns: Vec::new(),
        }
    }

    pub fn result(self) -> Vec<String> {
        self.exported_fns
    }
}

impl Fold for CollectExportedFnsVisitor {
    fn fold_module_decl(&mut self, node: swc_ecma_ast::ModuleDecl) -> ModuleDecl {
        match &node {
            ModuleDecl::ExportDecl(ExportDecl {
                decl: Decl::Fn(fn_decl),
                ..
            }) => self
                .exported_fns
                .push(parse_fn_name(fn_decl.ident.as_ref())),
            ModuleDecl::ExportDecl(ExportDecl {
                decl: Decl::Var(var_decl),
                ..
            }) => {
                for var_declarator in &var_decl.decls {
                    if let Some(expr) = &var_declarator.init {
                        if expr.as_arrow().is_some() || expr.as_fn_expr().is_some() {
                            self.exported_fns.push(
                                var_declarator
                                    .name
                                    .as_ident()
                                    .map(|a| parse_fn_name(a.as_ref()))
                                    .unwrap(),
                            )
                        }
                    }
                }
            }
            _ => {}
        }
        node
    }
}

fn parse_fn_name(s: &str) -> String {
    s.trim_end_matches(char::is_numeric)
        .trim_end_matches(char::is_numeric)
        .trim_end_matches('#')
        .to_string()
}
