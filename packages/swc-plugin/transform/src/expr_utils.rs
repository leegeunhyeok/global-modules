use swc_core::{
    common::DUMMY_SP,
    ecma::{ast::*, utils::ExprFactory},
};

pub fn get_require_expr(require_ident: &Ident, src: &Expr) -> Expr {
    require_ident
        .clone()
        .as_call(DUMMY_SP, vec![src.clone().as_arg()])
}
