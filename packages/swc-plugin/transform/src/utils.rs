use core::panic;

use swc_core::{
    atoms::Atom,
    common::DUMMY_SP,
    ecma::{ast::*, utils::ExprFactory},
};

/// ```js
/// var { foo, bar, default: baz } = __require('./foo'); // is_ns: false
/// var { foo, bar, default: baz } = __require('./foo', true); // is_ns: true
/// ```
pub fn get_require_call_stmt(
    require_ident: &Ident,
    src: &Atom,
    pat: Pat,
    is_ns: bool,
) -> ModuleItem {
    get_require_expr(require_ident, src, is_ns)
        .into_var_decl(VarDeclKind::Var, pat)
        .into()
}

/// ```js
/// __require('./foo', is_star);
/// ```
pub fn get_require_expr(require_ident: &Ident, src: &Atom, is_star: bool) -> Expr {
    let mut require_args = vec![src.clone().as_arg()];

    if is_star {
        require_args.push(
            Lit::Bool(Bool {
                span: DUMMY_SP,
                value: true,
            })
            .as_arg(),
        );
    }

    require_ident.clone().as_call(DUMMY_SP, require_args)
}

pub fn get_expr_from_decl(decl: &Decl) -> (Ident, Expr) {
    match decl {
        Decl::Class(ClassDecl {
            class,
            ident,
            declare: false,
        }) => (
            ident.clone(),
            ClassExpr {
                class: class.clone(),
                ident: Some(ident.clone()),
            }
            .into(),
        ),
        Decl::Fn(FnDecl {
            function,
            ident,
            declare: false,
        }) => (
            ident.clone(),
            FnExpr {
                function: function.clone(),
                ident: Some(ident.clone()),
            }
            .into(),
        ),
        Decl::Var(val_decl) => {
            if val_decl.decls.len() != 1 {
                panic!("invalid named exports");
            }

            let var_decl = val_decl.decls.get(0).unwrap();

            match var_decl {
                VarDeclarator {
                    name: Pat::Ident(BindingIdent { id, type_ann: None }),
                    init: Some(expr),
                    definite: false,
                    ..
                } => (id.clone(), *expr.clone()),
                _ => panic!("invalid"),
            }
        }
        _ => panic!("invalid"),
    }
}

pub fn get_expr_from_default_decl(default_decl: &DefaultDecl) -> Expr {
    match default_decl {
        DefaultDecl::Class(class_expr) => Expr::Class(class_expr.clone()),
        DefaultDecl::Fn(fn_expr) => Expr::Fn(fn_expr.clone()),
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

pub fn get_assign_expr(ident: Ident, expr: Expr) -> AssignExpr {
    AssignExpr {
        left: AssignTarget::Simple(SimpleAssignTarget::Ident(ident.into())),
        right: expr.into(),
        op: AssignOp::Assign,
        ..Default::default()
    }
}
