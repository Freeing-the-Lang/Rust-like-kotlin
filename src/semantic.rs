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

    // ★ 출력 기능
    Println(IRExpr),
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
    functions: Vec<Function>,
    map: HashMap<String, Function>,

    // builtin 함수 목록
    pub builtins: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new(program: Program) -> Self {
        let mut map = HashMap::new();
        for f in &program.funcs {
            map.insert(f.name.clone(), f.clone());
        }

        Self {
            functions: program.funcs,
            map,
            builtins: vec!["println".to_string()],
        }
    }

    pub fn analyze(&self) -> IRProgram {
        let mut funcs = Vec::new();
        for f in &self.functions {
            funcs.push(self.analyze_function(f));
        }
        IRProgram { funcs }
    }

    fn analyze_function(&self, f: &Function) -> IRFunction {
        let mut scope: HashMap<String, TypeName> = HashMap::new();

        for (pname, ptype) in &f.params {
            scope.insert(pname.clone(), ptype.clone());
        }

        let mut ir_body = Vec::new();
        for stmt in &f.body {
            let items = self.analyze_stmt(stmt, &mut scope, &f.ret_type);
            ir_body.extend(items);
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
                let et = self.expr_type(expr, scope);
                if &et != t {
                    panic!("Type error: expected {:?}, got {:?}", t, et);
                }
                let e = self.analyze_expr(expr, scope);
                scope.insert(name.clone(), t.clone());
                vec![IR::StoreVar(name.clone(), e)]
            }

            Stmt::Return(expr) => {
                let et = self.expr_type(expr, scope);
                if &et != expected_ret {
                    panic!("Return type mismatch");
                }
                let e = self.analyze_expr(expr, scope);
                vec![IR::Return(e)]
            }

            Stmt::ExprStmt(expr) => {
                // builtin println 변환
                if let Expr::Call(name, args) = expr {
                    if self.builtins.contains(name) {
                        if args.len() != 1 {
                            panic!("println expects 1 argument");
                        }
                        let arg_t = self.expr_type(&args[0], scope);
                        if arg_t != TypeName::String {
                            panic!("println expects String");
                        }
                        let e = self.analyze_expr(&args[0], scope);
                        return vec![IR::Println(e)];
                    }
                }

                // 일반 표현식문은 그냥 IR 저장
                let e = self.analyze_expr(expr, scope);
                vec![IR::StoreVar("_expr_tmp".to_string(), e)]
            }

            Stmt::If(cond, then_body, else_body) => {
                let ct = self.expr_type(cond, scope);
                if ct != TypeName::Int {
                    panic!("If condition must be int");
                }

                let cond_ir = self.analyze_expr(cond, scope);

                let mut tvec = Vec::new();
                for s in then_body {
                    tvec.extend(self.analyze_stmt(s, scope, expected_ret));
                }

                let mut evec = Vec::new();
                for s in else_body {
                    evec.extend(self.analyze_stmt(s, scope, expected_ret));
                }

                vec![IR::If(Box::new(cond_ir), tvec, evec)]
            }
        }
    }

    fn analyze_expr(&self, expr: &Expr, scope: &HashMap<String, TypeName>) -> IRExpr {
        match expr {
            Expr::Number(n) => IRExpr::Int(*n),
            Expr::StringLiteral(s) => IRExpr::Str(s.clone()),
            Expr::Var(name) => IRExpr::Var(name.clone()),

            Expr::Binary(a, op, b) => {
                IRExpr::Binary(
                    Box::new(self.analyze_expr(a, scope)),
                    op.clone(),
                    Box::new(self.analyze_expr(b, scope)),
                )
            }

            Expr::Call(name, args) => {
                // builtin println 은 이미 stmt에서 처리됨
                if !self.map.contains_key(name) {
                    panic!("Unknown function {}", name);
                }

                let func = self.map.get(name).unwrap();
                if func.params.len() != args.len() {
                    panic!("Argument count mismatch");
                }

                let mut ir_args = Vec::new();
                for (i, a) in args.iter().enumerate() {
                    let at = self.expr_type(a, scope);
                    let pt = &func.params[i].1;
                    if at != *pt {
                        panic!("Argument type mismatch");
                    }
                    ir_args.push(self.analyze_expr(a, scope));
                }

                IRExpr::Call(name.clone(), ir_args)
            }
        }
    }

    fn expr_type(&self, expr: &Expr, scope: &HashMap<String, TypeName>) -> TypeName {
        match expr {
            Expr::Number(_) => TypeName::Int,
            Expr::StringLiteral(_) => TypeName::String,

            Expr::Var(name) => scope.get(name).unwrap().clone(),

            Expr::Binary(a, op, b) => {
                let lt = self.expr_type(a, scope);
                let rt = self.expr_type(b, scope);

                if op == "+" && lt == TypeName::String && rt == TypeName::String {
                    return TypeName::String;
                }

                if lt != TypeName::Int || rt != TypeName::Int {
                    panic!("Binary op requires int");
                }

                TypeName::Int
            }

            Expr::Call(name, _) => {
                if self.builtins.contains(name) {
                    return TypeName::Int;
                }

                let func = self.map.get(name).unwrap();
                func.ret_type.clone()
            }
        }
    }
}
