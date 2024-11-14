use core::panic;

use swc_core::{
    atoms::Atom,
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::{ExprFactory, FunctionFactory},
    },
};

pub fn get_require_expr(require_ident: &Ident, src: &Atom) -> Expr {
    require_ident
        .clone()
        .as_call(DUMMY_SP, vec![src.clone().as_arg()])
}

pub fn get_expr_from_decl(decl: &Decl) -> Expr {
    match decl {
        Decl::Class(ClassDecl {
            class,
            ident,
            declare: false,
        }) => ClassExpr {
            class: class.clone(),
            ident: Some(ident.clone()),
        }
        .into(),
        Decl::Fn(FnDecl {
            function,
            ident,
            declare: false,
        }) => FnExpr {
            function: function.clone(),
            ident: Some(ident.clone()),
        }
        .into(),
        Decl::Var(val_decl) => {
            if val_decl.decls.len() != 1 {
                panic!("invalid named exports");
            }

            if let Some(init) = &val_decl.decls.get(0).unwrap().init {
                *init.clone()
            } else {
                panic!("invalid");
            }
        }
        _ => panic!("not implemented"),
    }
}

/// Wrap expression with function.
///
/// ```js
/// function () {
///   return /* expr */;
/// }
/// ```
pub fn wrap_with_fn(expr: &Expr) -> Expr {
    Function {
        body: Some(BlockStmt {
            stmts: vec![Stmt::Return(ReturnStmt {
                arg: Some(Box::new(expr.clone())),
                ..Default::default()
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
    .into()
}
