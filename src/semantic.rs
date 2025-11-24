use crate::parser::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum IR {
    LoadVar(String),
    StoreVar(String, IRExpr),
    LiteralInt(i64),
    LiteralString(String),
    BinaryOp(Box<IRExpr>, String, Box<IRExpr>),
    CallFunc(String, Vec<IRExpr>),
    If(Box<IRExpr>, Vec<IR>, Vec<IR>),
    Return(IRExpr),
}

#[derive(Debug, Clone)]
pub enum IRExpr {
    Var(String),
    Int(i64),
    Str(String),
    Binary(Box<IRExpr>, String, Box<IRExpr>),
    Call(String, Vec<IRExpr>),
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<(String, TypeName)>,
    pub ret_type: TypeName,
    pub body: Vec<IR>,
}

#[derive(Debug, Clone)]
pub struct IRProgram {
    pub funcs: Vec<IRFunction>,
}

pub struct SemanticAnalyzer {
    functions: HashMap<String, Function>,
}

impl SemanticAnalyzer {
    pub fn new(program: Program) -> Self {
        let mut functions = HashMap::new();
        for f in &program.funcs {
            functions.insert(f.name.clone(), f.clone());
        }

        Self { functions }
    }

    pub fn analyze(&self) -> IRProgram {
        let mut funcs = Vec::new();

        for (_, f) in &self.functions {
            funcs.push(self.analyze_function(f));
        }

        IRProgram { funcs }
    }

    fn analyze_function(&self, f: &Function) -> IRFunction {
        let mut scope: HashMap<String, TypeName> = HashMap::new();

        // Add params to local scope
        for (pname, ptype) in &f.params {
            scope.insert(pname.clone(), ptype.clone());
        }

        let mut ir_body = Vec::new();

        for stmt in &f.body {
            let ir = self.analyze_stmt(stmt, &mut scope, &f.ret_type);
            ir_body.extend(ir);
        }

        IRFunction {
            name: f.name.clone(),
            params: f.params.clone(),
            ret_type: f.ret_type.clone(),
            body: ir_body,
        }
    }

    fn analyze_stmt(
        &self,
        stmt: &Stmt,
        scope: &mut HashMap<String, TypeName>,
        expected_ret: &TypeName,
    ) -> Vec<IR> {
        match stmt {
            Stmt::Let(name, t, expr) => {
                let e = self.analyze_expr(expr, scope);

                // type checking
                let et = self.expr_type(expr, scope);

                if &et != t {
                    panic!(
                        "Type error: expected {:?} but got {:?} for variable {}",
                        t, et, name
                    );
                }

                scope.insert(name.clone(), t.clone());

                vec![IR::StoreVar(name.clone(), e)]
            }

            Stmt::Return(expr) => {
                let et = self.expr_type(expr, scope);

                if &et != expected_ret {
                    panic!(
                        "Return type mismatch: expected {:?} but got {:?}",
                        expected_ret, et
                    );
                }

                let e = self.analyze_expr(expr, scope);
                vec![IR::Return(e)]
            }

            Stmt::ExprStmt(expr) => {
                let e = self.analyze_expr(expr, scope);
                vec![IR::StoreVar("_expr_tmp".to_string(), e)]
            }

            Stmt::If(cond, then_body, else_body) => {
                // cond must be int
                let ct = self.expr_type(cond, scope);
                if ct != TypeName::Int {
                    panic!("If condition must be int, got {:?}", ct);
                }

                let cond_ir = self.analyze_expr(cond, scope);

                let mut then_ir = Vec::new();
                for s in then_body {
                    let ir = self.analyze_stmt(s, scope, expected_ret);
                    then_ir.extend(ir);
                }

                let mut else_ir = Vec::new();
                for s in else_body {
                    let ir = self.analyze_stmt(s, scope, expected_ret);
                    else_ir.extend(ir);
                }

                vec![IR::If(Box::new(cond_ir), then_ir, else_ir)]
            }
        }
    }

    fn analyze_expr(&self, expr: &Expr, scope: &HashMap<String, TypeName>) -> IRExpr {
        match expr {
            Expr::Number(n) => IRExpr::Int(*n),
            Expr::StringLiteral(s) => IRExpr::Str(s.clone()),
            Expr::Var(name) => IRExpr::Var(name.clone()),

            Expr::Binary(a, op, b) => {
                let left = self.analyze_expr(a, scope);
                let right = self.analyze_expr(b, scope);
                IRExpr::Binary(Box::new(left), op.clone(), Box::new(right))
            }

            Expr::Call(name, args) => {
                if !self.functions.contains_key(name) {
                    panic!("Unknown function '{}'", name);
                }

                let func = self.functions.get(name).unwrap();

                if func.params.len() != args.len() {
                    panic!(
                        "Argument count mismatch in call {}: expected {}, got {}",
                        name,
                        func.params.len(),
                        args.len()
                    );
                }

                // type check args
                for (i, arg_expr) in args.iter().enumerate() {
                    let arg_t = self.expr_type(arg_expr, scope);
                    let param_t = &func.params[i].1;

                    if &arg_t != param_t {
                        panic!(
                            "Type mismatch in argument {} of function {}: expected {:?}, got {:?}",
                            i, name, param_t, arg_t
                        );
                    }
                }

                let mut ir_args = Vec::new();
                for arg in args {
                    ir_args.push(self.analyze_expr(arg, scope));
                }

                IRExpr::Call(name.clone(), ir_args)
            }
        }
    }

    fn expr_type(&self, expr: &Expr, scope: &HashMap<String, TypeName>) -> TypeName {
        match expr {
            Expr::Number(_) => TypeName::Int,
            Expr::StringLiteral(_) => TypeName::String,

            Expr::Var(name) => scope
                .get(name)
                .unwrap_or_else(|| panic!("Unknown variable: {}", name))
                .clone(),

            Expr::Binary(a, op, b) => {
                let lt = self.expr_type(a, scope);
                let rt = self.expr_type(b, scope);

                if op == "+" && lt == TypeName::String && rt == TypeName::String {
                    return TypeName::String;
                }

                // all other operators expect int
                if lt != TypeName::Int || rt != TypeName::Int {
                    panic!("Operator '{}' requires int operands", op);
                }

                TypeName::Int
            }

            Expr::Call(name, _) => {
                let func = self
                    .functions
                    .get(name)
                    .unwrap_or_else(|| panic!("Function not found: {}", name));
                func.ret_type.clone()
            }
        }
    }
                      }
