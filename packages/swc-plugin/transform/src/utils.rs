pub mod ast {
    use crate::models::*;
    use core::panic;
    use swc_core::{
        atoms::Atom,
        common::{collections::AHashMap, Spanned, SyntaxContext, DUMMY_SP},
        ecma::{
            ast::*,
            utils::{private_ident, ExprFactory},
        },
        plugin::errors::HANDLER,
    };

    /// Returns a binding identifier for the default export.
    ///
    /// ```js
    /// // Code
    /// __default;
    /// ```
    pub fn anonymous_default_binding_ident() -> Ident {
        private_ident!("__default")
    }

    /// Returns a binding identifier.
    ///
    /// ```js
    /// // Code
    /// __x;
    /// ```
    pub fn exp_binding_ident() -> Ident {
        private_ident!("__x")
    }

    /// Returns a module identifier.
    ///
    /// ```js
    /// // Code
    /// __mod;
    /// ```
    pub fn mod_ident() -> Ident {
        private_ident!("__mod")
    }

    /// Returns a key-value property.
    /// Can be used to create a assign expression.
    ///
    /// ```js
    /// // Code
    /// var { key: value } = value;
    /// ```
    pub fn kv_prop(key: Atom, value: Expr) -> PropOrSpread {
        PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Str(Str {
                value: key,
                raw: None,
                span: DUMMY_SP,
            }),
            value: value.into(),
        })))
    }

    /// Returns a spread property.
    /// Can be used to create a assign expression.
    ///
    /// ```js
    /// // Code
    /// var { ...expr } = value;
    /// ```
    pub fn spread_prop(expr: Expr) -> PropOrSpread {
        PropOrSpread::Spread(SpreadElement {
            expr: expr.into(),
            ..Default::default()
        })
    }

    /// Returns an object key-value property.
    /// Can be used to create a object literal expression.
    ///
    /// ```js
    /// // Code
    /// var value = { key: value };
    /// ```
    pub fn obj_kv_prop(key: Ident, value: Ident) -> ObjectPatProp {
        ObjectPatProp::KeyValue(KeyValuePatProp {
            key: PropName::Ident(key.into()),
            value: Box::new(Pat::Ident(value.into())),
        })
    }

    /// Returns an object assign property.
    /// Can be used to create a object literal expression.
    ///
    /// ```js
    /// // Code
    /// var value = { key };
    /// ```
    pub fn obj_assign_prop(key: Ident) -> ObjectPatProp {
        ObjectPatProp::Assign(AssignPatProp {
            key: key.into(),
            value: None,
            span: DUMMY_SP,
        })
    }

    /// Returns an assign expression.
    ///
    /// ```js
    /// // Code
    /// ident = expr;
    /// ```
    pub fn assign_expr(ident: Ident, expr: Expr) -> AssignExpr {
        AssignExpr {
            left: AssignTarget::Simple(SimpleAssignTarget::Ident(ident.into())),
            right: expr.into(),
            op: AssignOp::Assign,
            ..Default::default()
        }
    }

    /// Returns an object literal expression.
    ///
    /// ```js
    /// // Code
    /// { prop: value }
    /// ```
    pub fn obj_lit_expr(props: Vec<PropOrSpread>) -> Expr {
        Expr::Object(ObjectLit {
            props,
            ..Default::default()
        })
    }

    /// Returns a string literal expression.
    ///
    /// ```js
    /// // Code
    /// 'foo'
    /// ```
    pub fn str_lit(str: &String) -> Lit {
        Lit::Str(Str {
            value: str.as_str().into(),
            raw: None,
            span: DUMMY_SP,
        })
    }

    /// Returns a variable declarator bound to the provided identifier.
    ///
    /// ```js
    /// // Declarator
    /// // name, init
    ///
    /// // Code
    /// var name = init;
    /// ```
    pub fn var_declarator(name: Pat, init: Option<Box<Expr>>) -> VarDeclarator {
        VarDeclarator {
            name,
            init,
            definite: false,
            span: DUMMY_SP,
        }
    }

    /// Returns an import-all statement bound to the provided identifier.
    ///
    /// ```js
    /// // Code
    /// import * as ident from 'src';
    /// ```
    pub fn import_star(ident: Ident, src: Atom) -> ModuleItem {
        ModuleDecl::Import(ImportDecl {
            phase: ImportPhase::Evaluation,
            specifiers: vec![ImportSpecifier::Namespace(ImportStarAsSpecifier {
                local: ident,
                span: DUMMY_SP,
            })],
            src: Box::new(src.into()),
            type_only: false,
            with: None,
            span: DUMMY_SP,
        })
        .into()
    }

    /// Wraps the given expression with a `ctx_ident.exports.ns` function call expression.
    ///
    /// ```js
    /// // Code
    /// ctx_ident.exports.ns(expr);
    /// ```
    pub fn to_ns_export(ctx_ident: Ident, expr: Expr) -> Expr {
        ctx_ident
            .make_member("exports".into())
            .make_member("ns".into())
            .as_call(DUMMY_SP, vec![expr.into()])
    }

    /// Checks whether it is a CommonJS `require` function call.
    ///
    /// ```js
    /// // Case 1.
    /// require('src'); // true
    ///
    /// // Case 2.
    /// function foo(require) {
    ///   require('src'); // false
    /// }
    /// ```
    pub fn is_require_call(unresolved_ctxt: SyntaxContext, call_expr: &CallExpr) -> bool {
        // `require` call must have exactly one argument
        if call_expr.args.len() != 1 {
            return false;
        }

        match &call_expr.callee {
            Callee::Expr(callee_expr) => {
                // Check callee name is `require` and its context is unresolved (global identifier)
                callee_expr.is_ident_ref_to("require")
                    && callee_expr.as_ident().unwrap().ctxt == unresolved_ctxt
            }
            _ => false,
        }
    }

    /// Checks whether it is a member expression of a CommonJS module.
    ///
    /// ```js
    /// // Code
    /// exports.foo; // true;
    /// ```
    pub fn is_cjs_exports_member(unresolved_ctxt: SyntaxContext, member_expr: &MemberExpr) -> bool {
        // Check object is `exports` and its context is unresolved (global identifier)
        member_expr.obj.is_ident_ref_to("exports")
            && member_expr.obj.as_ident().unwrap().ctxt == unresolved_ctxt
    }

    /// Checks whether it is a member expression of a CommonJS module.
    ///
    /// ```js
    /// // Code
    /// module.exports; // true;
    /// ```
    pub fn is_cjs_module_member(unresolved_ctxt: SyntaxContext, member_expr: &MemberExpr) -> bool {
        // Check object is `module` and its context is unresolved (global identifier)
        member_expr.obj.is_ident_ref_to("module")
            && member_expr.obj.as_ident().unwrap().ctxt == unresolved_ctxt
            && member_expr.prop.is_ident_with("exports")
    }

    /// Returns a new expression that assigns to a member expression.
    ///
    /// ```js
    /// // Code
    /// member.prop = right_expr;
    /// ```
    pub fn assign_member(left: MemberExpr, right: Expr) -> Expr {
        right.make_assign_to(
            AssignOp::Assign,
            AssignTarget::Simple(SimpleAssignTarget::Member(left)),
        )
    }

    /// Returns an arrow function expression with a parenthesized expression.
    ///
    /// ```js
    /// // Code
    /// () => (expr);
    /// ```
    pub fn arrow_with_paren_expr(expr: Expr) -> Expr {
        Expr::Arrow(ArrowExpr {
            body: Box::new(BlockStmtOrExpr::Expr(Box::new(Expr::Paren(ParenExpr {
                span: DUMMY_SP,
                expr: Box::new(expr),
            })))),
            ..Default::default()
        })
    }

    /// Extracts and returns the its ident from the declarations.
    ///
    /// ```js
    /// // Code
    /// function foo {} // foo
    /// class Bar {} // Bar
    /// const baz = expr; // baz
    /// ```
    pub fn get_ident_from_decl(decl: &Decl) -> Option<Ident> {
        match decl {
            Decl::Class(ClassDecl {
                ident,
                declare: false,
                ..
            }) => Some(ident.clone()),
            Decl::Fn(FnDecl {
                ident,
                declare: false,
                ..
            }) => Some(ident.clone()),
            Decl::Var(val_decl) => {
                if val_decl.decls.len() != 1 {
                    HANDLER.with(|handler| {
                        handler
                            .struct_span_err(
                                val_decl.span,
                                "multiple variable declarations are not allowed",
                            )
                            .emit();
                    });
                    panic!(); // FIXME
                }

                let var_decl = val_decl.decls.get(0).unwrap();

                match var_decl {
                    VarDeclarator {
                        name: Pat::Ident(BindingIdent { id, type_ann: None }),
                        definite: false,
                        ..
                    } => Some(id.clone()),
                    _ => {
                        HANDLER.with(|handler| {
                            handler
                                .struct_span_err(var_decl.span, "unsupported variable declaration")
                                .emit();
                        });
                        panic!(); // FIXME
                    }
                }
            }
            _ => None,
        }
    }

    /// Returns a mapped source string from the given literal.
    ///
    /// When lit argument is `Lit::Str(Str { value: 'src' })`:
    /// - Case 1. No matches key in `deps_id`
    ///   - Returns `"src"`
    /// - Case 2. Matches key in `deps_id` ({ "src": "override_src" })
    ///   - Returns `"override_src"`
    pub fn get_src(lit: &Lit, deps_id: &Option<AHashMap<String, String>>) -> String {
        match lit {
            Lit::Str(Str { value, .. }) => {
                let src = value.to_string();
                deps_id
                    .as_ref()
                    .and_then(|deps_id| deps_id.get(src.as_str()))
                    .map_or_else(|| src, |id| id.as_str().to_string())
            }
            _ => HANDLER.with(|handler| {
                handler
                    .struct_span_err(lit.span(), "unsupported literal type")
                    .emit();

                panic!()
            }),
        }
    }

    /// Returns an expression that represents a CommonJS module export name.
    ///
    /// ```js
    /// // Given code
    /// exports.foo; // Returns "foo"
    /// ```
    pub fn to_cjs_export_name(prop: &MemberProp) -> Expr {
        match prop {
            MemberProp::Ident(ident) => Expr::Lit(Lit::Str(Str {
                value: ident.sym.as_str().into(),
                raw: None,
                span: DUMMY_SP,
            })),
            MemberProp::Computed(ComputedPropName { expr, .. }) => match &**expr {
                str_lit_expr @ Expr::Lit(Lit::Str(_)) => str_lit_expr.clone(),
                _ => *expr.clone(),
            },
            _ => HANDLER.with(|handler| {
                handler
                    .struct_span_err(prop.span(), "unsupported property type")
                    .emit();

                panic!("fatal error");
            }),
        }
    }

    /// Converts an import declaration to a `Dep`.
    pub fn import_as_dep(import_decl: &ImportDecl) -> Option<Dep> {
        let src = import_decl.src.value.to_string();
        let members = import_decl
            .specifiers
            .iter()
            .filter_map(|spec| match spec {
                // Named import
                //
                // ```js
                // import { foo, bar as baz } from 'src';
                // ```
                ImportSpecifier::Named(ImportNamedSpecifier {
                    imported,
                    local,
                    is_type_only: false,
                    ..
                }) => Some(DepMember::new(
                    local.clone(),
                    imported.as_ref().map(|name| match name {
                        ModuleExportName::Ident(ident) => ident.sym.as_str().to_string(),
                        ModuleExportName::Str(str) => str.value.to_string(),
                    }),
                )),
                // Default import
                //
                // ```js
                // import foo from 'src';
                // ```
                ImportSpecifier::Default(ImportDefaultSpecifier { local, .. }) => {
                    Some(DepMember::new(local.clone(), Some("default".into())))
                }
                // Namespace import
                //
                // ```js
                // import * as foo from 'src';
                // ```
                ImportSpecifier::Namespace(ImportStarAsSpecifier { local, .. }) => {
                    Some(DepMember::new(local.clone(), None))
                }
                _ => None,
            })
            .collect::<Vec<DepMember>>();

        // If there are no members, return None
        if members.is_empty() {
            None
        } else {
            Some(Dep::base(src, members))
        }
    }

    /// Converts an export declaration to an `Exp`.
    pub fn export_decl_as_exp(export_decl: &ExportDecl) -> Option<(Exp, Stmt, ExpBinding)> {
        // When export declaration has a own identifier.
        if let Some(decl_ident) = get_ident_from_decl(&export_decl.decl) {
            let exp_binding_ident = exp_binding_ident();
            let name = decl_ident.sym.as_str().to_string();
            let exp = Exp::Base(BaseExp::new(vec![ExpMember::new(
                exp_binding_ident.clone(),
                name,
            )]));

            Some((
                exp,
                // Keep the original export declaration
                Stmt::Decl(export_decl.decl.clone()),
                // Create binding to reference the export declaration's identifier
                //
                // ```js
                // // Given code
                // export function foo() {}
                // ```
                // - binding_ident: __x
                // - expr: foo (decl_ident)
                ExpBinding {
                    binding_ident: exp_binding_ident,
                    expr: decl_ident.into(),
                },
            ))
        } else {
            None
        }
    }

    /// Converts an export default declaration to an `Exp`.
    pub fn export_default_decl_as_exp(
        export_default_decl: &ExportDefaultDecl,
    ) -> Option<(Exp, Decl, ExpBinding)> {
        if let Some(decl) = match &export_default_decl.decl {
            DefaultDecl::Class(class_expr) => {
                // Clone class identifier if has one or create a new anonymous identifier for bind
                let class_ident = class_expr
                    .ident
                    .clone()
                    .unwrap_or_else(|| anonymous_default_binding_ident());

                Some(Decl::Class(ClassDecl {
                    ident: class_ident.clone(),
                    class: class_expr.class.clone(),
                    declare: false,
                }))
            }
            DefaultDecl::Fn(fn_expr) => {
                // Clone function identifier if has one or create a new anonymous identifier for bind
                let fn_ident = fn_expr
                    .ident
                    .clone()
                    .unwrap_or_else(|| anonymous_default_binding_ident());

                Some(Decl::Fn(FnDecl {
                    ident: fn_ident,
                    function: fn_expr.function.clone(),
                    declare: false,
                }))
            }
            // Ignore TypeScript interface declaration
            DefaultDecl::TsInterfaceDecl(_) => None,
        } {
            let decl_ident = match &decl {
                Decl::Class(class_decl) => class_decl.ident.clone(),
                Decl::Fn(fn_decl) => fn_decl.ident.clone(),
                _ => unreachable!(),
            };
            let exp_binding_ident = exp_binding_ident();
            let exp = Exp::Base(BaseExp::new(vec![ExpMember::new(
                exp_binding_ident.clone(),
                "default".into(),
            )]));

            Some((
                exp,
                // Keep the original export default declaration
                decl,
                // Create binding to reference the export declaration's identifier
                //
                // ```js
                // // Given code
                // export default function() {} // No identifier. It will be created an anonymous identifier for binding
                // ```
                // - binding_ident: __x
                // - expr: __default (decl_ident)
                ExpBinding {
                    binding_ident: exp_binding_ident,
                    expr: decl_ident.into(),
                },
            ))
        } else {
            None
        }
    }

    /// Converts an export default expression to an `Exp`.
    pub fn export_default_expr_as_exp(
        export_default_expr: &mut ExportDefaultExpr,
    ) -> (Exp, Stmt, ExpBinding) {
        let binding_ident = anonymous_default_binding_ident();
        let exp_binding_ident = exp_binding_ident();
        let exp = Exp::Base(BaseExp::new(vec![ExpMember::new(
            exp_binding_ident.clone(),
            "default".into(),
        )]));

        // Create a variable declaration to bind the default export expression
        //
        // ```js
        // // Given code
        // export default foo;
        //
        // // Returns
        // var __default = foo; // Use `__default` as binding identifier
        // ```
        let default_var_decl = VarDecl {
            decls: vec![var_declarator(
                binding_ident.clone().into(),
                Some(Box::new(*export_default_expr.expr.clone())),
            )],
            kind: VarDeclKind::Const,
            ..Default::default()
        };

        (
            exp,
            // Replace the original export default expression with a variable declaration
            default_var_decl.into(),
            // Create binding to reference the default export expression
            //
            // ```js
            // // Given code
            // // export default foo;
            // var __default = foo;
            // ```
            // - binding_ident: __x
            // - expr: __default (binding_ident)
            ExpBinding {
                binding_ident: exp_binding_ident,
                expr: binding_ident.into(),
            },
        )
    }

    /// Converts an export named declaration to an `Exp`.
    pub fn export_named_as_exp(export_named: &NamedExport) -> Option<(Exp, Vec<ExpBinding>)> {
        let mut exp_bindings: Vec<ExpBinding> = Vec::new();

        // If namespace export, it always has one specifier
        if let Some(specifier) = export_named.specifiers.get(0) {
            if specifier.is_namespace() {
                let ns = specifier.as_namespace().unwrap();
                let ident = match &ns.name {
                    ModuleExportName::Ident(ident) => ident.clone(),
                    ModuleExportName::Str(str) => {
                        Ident::new(str.value.clone(), DUMMY_SP, SyntaxContext::default())
                    }
                };

                return Some((
                    Exp::ReExportAll(ReExportAllExp::alias(
                        export_named.src.as_ref().unwrap().clone().value.to_string(),
                        ident,
                    )),
                    exp_bindings,
                ));
            }
        }

        // Otherwise, it has multiple specifiers (Non-namespace export)
        let members = export_named
            .specifiers
            .iter()
            .filter_map(|spec| match spec {
                // Default export
                //
                // ```js
                // export value from 'src';
                // ```
                ExportSpecifier::Default(default) => {
                    let exp_binding_ident = exp_binding_ident();

                    exp_bindings.push(ExpBinding {
                        binding_ident: exp_binding_ident.clone(),
                        expr: Expr::from(default.exported.clone()),
                    });

                    Some(ExpMember::new(exp_binding_ident, "default".into()))
                }
                // Named export
                //
                // ```js
                // // Named export
                // export { foo, bar as baz };
                // export { value as default };
                //
                // // Re-export
                // export { foo, bar as baz } from 'src';
                // export { value as default } from 'src';
                // ```
                ExportSpecifier::Named(ExportNamedSpecifier {
                    orig,
                    exported,
                    is_type_only: false,
                    ..
                }) => {
                    let exp_binding_ident = exp_binding_ident();
                    let exported_ident = match orig {
                        ModuleExportName::Ident(ident) => ident.clone(),
                        ModuleExportName::Str(str) => {
                            Ident::new(str.value.clone(), DUMMY_SP, SyntaxContext::default())
                        }
                    };
                    let name = exported.as_ref().map_or_else(
                        || exported_ident.sym.as_str().to_string(),
                        |name| match name {
                            ModuleExportName::Ident(ident) => ident.sym.as_str().to_string(),
                            ModuleExportName::Str(str) => str.value.to_string(),
                        },
                    );

                    if export_named.src.is_none() {
                        exp_bindings.push(ExpBinding {
                            binding_ident: exp_binding_ident.clone(),
                            expr: Expr::from(exported_ident),
                        });
                        Some(ExpMember::new(exp_binding_ident, name))
                    } else {
                        Some(ExpMember::new(exported_ident, name))
                    }
                }
                // Namespace export is already handled from the above condition
                ExportSpecifier::Namespace(_) => unreachable!(),
                _ => None,
            })
            .collect::<Vec<ExpMember>>();

        // If there are no members, return None
        if members.is_empty() {
            None
        } else {
            Some((
                if export_named.src.is_none() {
                    // Plain named export
                    Exp::Base(BaseExp::new(members))
                } else {
                    // Named re-export
                    Exp::ReExportNamed(ReExportNamedExp {
                        src: export_named.src.as_ref().unwrap().clone().value.to_string(),
                        members,
                    })
                },
                exp_bindings,
            ))
        }
    }

    /// Converts an export all declaration to an `Exp`.
    pub fn export_all_as_exp(export_all: &ExportAll) -> Exp {
        Exp::ReExportAll(ReExportAllExp::new(
            export_all.src.as_ref().clone().value.to_string(),
        ))
    }

    pub mod presets {
        use swc_core::{
            common::DUMMY_SP,
            ecma::{
                ast::*,
                utils::{member_expr, quote_ident, ExprFactory},
            },
        };

        use super::{arrow_with_paren_expr, assign_member, obj_lit_expr, str_lit, var_declarator};

        /// Returns a global module's require call expression.
        ///
        /// ```js
        /// // Code
        /// ctx_ident.require(src);
        /// ```
        pub fn require_call(ctx_ident: &Ident, src: Lit) -> Expr {
            ctx_ident
                .clone()
                .make_member(quote_ident!("require"))
                .as_call(DUMMY_SP, vec![src.as_arg()])
        }

        /// Returns a global module's exports call expression.
        ///
        /// ```js
        /// // Code
        /// ctx_ident.exports(function () {
        ///   return expr;
        /// });
        /// ```
        pub fn exports_call(ctx_ident: &Ident, exp_props: Vec<PropOrSpread>) -> Expr {
            ctx_ident
                .clone()
                .make_member(quote_ident!("exports"))
                .as_call(
                    DUMMY_SP,
                    vec![obj_lit_expr(exp_props).into_lazy_fn(vec![]).as_arg()],
                )
        }

        /// Returns a context module's exports member expression.
        ///
        /// ```js
        /// // Code
        /// ctx_ident.module.exports;
        /// ```
        pub fn module_exports_member(ctx_ident: &Ident) -> MemberExpr {
            ctx_ident
                .clone()
                .make_member(IdentName {
                    sym: "module".into(),
                    ..Default::default()
                })
                .make_member(IdentName {
                    sym: "exports".into(),
                    ..Default::default()
                })
        }

        /// Returns a new expression that binds the CommonJS module export statement.
        ///
        /// ```js
        /// // Code
        /// ctx_ident.module.exports.name = expr;
        /// ```
        pub fn assign_cjs_module_expr(ctx_ident: &Ident, expr: Expr, name: Option<Expr>) -> Expr {
            let ctx_module_member = module_exports_member(ctx_ident);

            assign_member(
                match name {
                    // Named export
                    //
                    // ```js
                    // module.exports.foo = expr;
                    // ```
                    Some(name_expr) => match name_expr {
                        Expr::Lit(Lit::Str(str_lit)) => ctx_module_member.make_member(IdentName {
                            sym: str_lit.value.clone().into(),
                            ..Default::default()
                        }),
                        _ => ctx_module_member.computed_member(name_expr.clone()),
                    },
                    // Main export
                    //
                    // ```js
                    // module.exports = expr;
                    // ```
                    None => ctx_module_member,
                },
                expr,
            )
            .into()
        }

        /// Returns a global module's define call expression.
        ///
        /// ```js
        /// // Code
        /// global.__modules.define(function (ctx_ident) {
        ///   // <stmts>
        /// }, id, deps_ident);
        /// ```
        pub fn define_call(
            id: &String,
            ctx_ident: &Ident,
            deps_ident: &Ident,
            stmts: Vec<Stmt>,
        ) -> Expr {
            member_expr!(Default::default(), DUMMY_SP, global.__modules.define).as_call(
                DUMMY_SP,
                vec![
                    Expr::Fn(FnExpr {
                        ident: None,
                        function: Box::new(Function {
                            params: vec![ctx_ident.clone().into()],
                            body: Some(BlockStmt {
                                stmts,
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .into(),
                    str_lit(id).as_arg(),
                    deps_ident.clone().as_arg(),
                ],
            )
        }

        /// Returns a named export statement based on given export specifiers.
        ///
        /// ```js
        /// // Code
        /// export { foo, bar as baz };
        /// ```
        pub fn to_named_exps(exp_specs: Vec<ExportSpecifier>) -> ModuleItem {
            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(NamedExport {
                specifiers: exp_specs,
                type_only: false,
                src: None,
                with: None,
                span: DUMMY_SP,
            }))
        }

        /// Returns a dependency getter expression based on given dependency member properties.
        ///
        /// ```js
        /// // Given code (= `{ foo, bar: baz }`)
        /// const { foo, bar: baz } = value;
        ///
        /// // Returns
        /// () => ({ foo, bar: baz });
        /// ```
        pub fn to_dep_getter_expr(dep_member_props: &Vec<ObjectPatProp>) -> Expr {
            let dep_obj_props = dep_member_props
                .iter()
                .filter_map(|prop| match prop {
                    ObjectPatProp::KeyValue(KeyValuePatProp { key, value })
                        if matches!(&**value, Pat::Ident(_)) =>
                    {
                        match &**value {
                            Pat::Ident(ident) => Some(
                                Prop::KeyValue(KeyValueProp {
                                    key: key.clone(),
                                    value: Box::new(Expr::Ident(ident.clone().into())),
                                })
                                .into(),
                            ),
                            _ => None,
                        }
                    }
                    ObjectPatProp::Assign(AssignPatProp {
                        key, value: None, ..
                    }) => Some(Prop::Shorthand(key.clone().into()).into()),
                    _ => None,
                })
                .collect::<Vec<PropOrSpread>>();

            arrow_with_paren_expr(
                ObjectLit {
                    props: dep_obj_props,
                    ..Default::default()
                }
                .into(),
            )
        }

        /// Returns a dependency declaration based on given dependency identifier and dependency getter expressions.
        ///
        /// ```js
        /// // Code
        /// const dep_ident = {
        ///   "src_1": () => ({ foo }),
        ///   "src_2": () => ({ bar }),
        ///   "src_3": () => ({ baz }),
        /// };
        /// ```
        pub fn to_deps_decl(dep_ident: &Ident, dep_getters: Vec<(String, Expr)>) -> Decl {
            Decl::Var(Box::new(VarDecl {
                decls: vec![var_declarator(
                    dep_ident.clone().into(),
                    Some(Box::new(Expr::Object(ObjectLit {
                        props: dep_getters
                            .iter()
                            .map(|(src, expr)| {
                                PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                                    key: PropName::Str(src.clone().into()),
                                    value: Box::new(expr.clone()),
                                })))
                            })
                            .collect(),
                        ..Default::default()
                    }))),
                )],
                kind: VarDeclKind::Const,
                ..Default::default()
            }))
        }

        /// Returns an empty dependency declaration.
        ///
        /// ```js
        /// // Code
        /// const dep_ident = {};
        /// ```
        pub fn to_empty_deps_decl(dep_ident: &Ident) -> Decl {
            Decl::Var(Box::new(VarDecl {
                decls: vec![var_declarator(dep_ident.clone().into(), None)],
                ..Default::default()
            }))
        }
    }
}

pub mod collections {
    use swc_core::common::collections::AHashMap;

    /// Ordered HashMap
    #[derive(Debug)]
    pub struct OHashMap<K, V> {
        map: AHashMap<K, V>,
        keys: Vec<K>,
    }

    impl<K: std::cmp::Eq + std::hash::Hash + Clone, V> OHashMap<K, V> {
        pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
            self.map.get_mut(key)
        }

        pub fn insert(&mut self, key: &K, value: V) {
            if !self.map.contains_key(&key) {
                self.keys.push(key.clone());
            }
            self.map.insert(key.clone(), value);
        }

        pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
            self.keys
                .iter()
                .filter_map(move |key| self.map.get(key).map(|value| (key, value)))
        }
    }

    impl<K: std::cmp::Eq + std::hash::Hash + Clone, V> Default for OHashMap<K, V> {
        fn default() -> Self {
            Self {
                map: AHashMap::default(),
                keys: Vec::new(),
            }
        }
    }
}
