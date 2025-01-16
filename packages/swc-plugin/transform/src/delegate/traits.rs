use swc_core::ecma::ast::*;

pub trait AstDelegate {
    fn make_script_body(&mut self, orig_body: Vec<Stmt>) -> Vec<Stmt>;
    fn make_module_body(&mut self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem>;
    fn import(&mut self, import_decl: &mut ImportDecl);
    fn export_decl(&mut self, export_decl: &mut ExportDecl) -> ModuleItem;
    fn export_default_decl(
        &mut self,
        export_default_decl: &mut ExportDefaultDecl,
    ) -> Option<ModuleItem>;
    fn export_default_expr(
        &mut self,
        export_default_expr: &mut ExportDefaultExpr,
    ) -> Option<ModuleItem>;
    fn export_named(&mut self, export_named: &mut NamedExport);
    fn export_all(&mut self, export_all: &mut ExportAll);
    fn call_expr(&mut self, call_expr: &mut CallExpr) -> Option<Expr>;
    fn assign_expr(&mut self, assign_expr: &mut AssignExpr) -> Option<Expr>;
    fn member_expr(&mut self, member_expr: &mut MemberExpr) -> Option<Expr>;
}
