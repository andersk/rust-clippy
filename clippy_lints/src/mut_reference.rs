use rustc::lint::*;
use rustc::ty::{TypeAndMut, TypeVariants, TyS};
use rustc::ty::subst::Subst;
use rustc::hir::*;
use utils::span_lint;

/// **What it does:** Detects giving a mutable reference to a function that only
/// requires an immutable reference.
///
/// **Why is this bad?** The immutable reference rules out all other references
/// to the value. Also the code misleads about the intent of the call site.
///
/// **Known problems:** None.
///
/// **Example:**
/// ```rust
/// my_vec.push(&mut value)
/// ```
declare_lint! {
    pub UNNECESSARY_MUT_PASSED,
    Warn,
    "an argument passed as a mutable reference although the callee only demands an \
     immutable reference"
}


#[derive(Copy,Clone)]
pub struct UnnecessaryMutPassed;

impl LintPass for UnnecessaryMutPassed {
    fn get_lints(&self) -> LintArray {
        lint_array!(UNNECESSARY_MUT_PASSED)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for UnnecessaryMutPassed {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, e: &'tcx Expr) {
        match e.node {
            ExprCall(ref fn_expr, ref arguments) => {
                if let ExprPath(ref path) = fn_expr.node {
                    check_arguments(cx,
                                    arguments,
                                    cx.tables.expr_ty(fn_expr),
                                    &print::to_string(print::NO_ANN, |s| s.print_qpath(path, false)));
                }
            },
            ExprMethodCall(ref name, _, ref arguments) => {
                let def_id = cx.tables.type_dependent_defs[&e.id].def_id();
                let substs = cx.tables.node_substs(e.id);
                let method_type = cx.tcx.type_of(def_id).subst(cx.tcx, substs);
                check_arguments(cx, arguments, method_type, &name.node.as_str())
            },
            _ => (),
        }
    }
}

fn check_arguments(cx: &LateContext, arguments: &[Expr], type_definition: &TyS, name: &str) {
    match type_definition.sty {
        TypeVariants::TyFnDef(_, _, fn_type) |
        TypeVariants::TyFnPtr(fn_type) => {
            let parameters = fn_type.skip_binder().inputs();
            for (argument, parameter) in arguments.iter().zip(parameters.iter()) {
                match parameter.sty {
                    TypeVariants::TyRef(_, TypeAndMut { mutbl: MutImmutable, .. }) |
                    TypeVariants::TyRawPtr(TypeAndMut { mutbl: MutImmutable, .. }) => {
                        if let ExprAddrOf(MutMutable, _) = argument.node {
                            span_lint(cx,
                                      UNNECESSARY_MUT_PASSED,
                                      argument.span,
                                      &format!("The function/method `{}` doesn't need a mutable reference", name));
                        }
                    },
                    _ => (),
                }
            }
        },
        _ => (),
    }
}
