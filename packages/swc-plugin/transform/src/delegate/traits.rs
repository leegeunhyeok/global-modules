use swc_core::ecma::ast::*;

pub trait AstDelegate {
    fn make_script_body(&mut self, orig_body: Vec<Stmt>) -> Vec<Stmt>;
    fn make_module_body(&mut self, orig_body: Vec<ModuleItem>) -> Vec<ModuleItem>;
    fn import(&mut self, import_decl: &ImportDecl);
    fn export_decl(&mut self, export_decl: &ExportDecl) -> ModuleItem;
    fn export_default_decl(&mut self, export_default_decl: &ExportDefaultDecl) -> ModuleItem;
    fn export_default_expr(&mut self, export_default_expr: &ExportDefaultExpr) -> Option<Expr>;
    fn export_named(&mut self, export_named: &NamedExport);
    fn export_all(&mut self, export_all: &ExportAll);
    fn call_expr(&mut self, call_expr: &CallExpr) -> Option<Expr>;
    fn assign_expr(&mut self, assign_expr: &AssignExpr) -> Option<Expr>;
    fn member_expr(&mut self, member_expr: &MemberExpr) -> Option<Expr>;
}
