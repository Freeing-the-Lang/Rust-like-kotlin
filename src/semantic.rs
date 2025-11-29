use crate::parser::*;
use std::collections::{HashMap, HashSet};

//
// Built-in functions (semantic only, no user definition needed)
//
const BUILTIN_FUNCS: &[&str] = &[
    "println",
    "print",
];

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

//
// Semantic Analyzer
//
pub struct SemanticAnalyzer {
    // 전체 Function 리스트 (순서를 유지)
    functions: Vec<Function>,

    // 이름 → Function
    map: HashMap<String, Function>,

    // built-in 함수 집합
    builtins: HashSet<String>,
}

impl SemanticAnalyzer {
    pub fn new(program: Program) -> Self {
        let mut map = HashMap::new();
        for f in &program.funcs {
            map.insert(f.name.clone(), f.clone());
        }

        let mut builtins = HashSet::new();
        for b in BUILTIN_FUNCS {
            builtins.insert((*b).to_string());
        }

        Self {
            functions: program.funcs,
            map,
            builtins,
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

        // 파라미터 → 스코프 등록
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
            //
            // let name: Type = expr;
            //
            Stmt::Let(name, t, expr) => {
                let et = self.expr_type(expr, scope);

                if &et != t {
                    panic!(
                        "Type error: expected {:?} but got {:?} for variable {}",
                        t, et, name
                    );
                }

                let e = self.analyze_expr(expr, scope);
                scope.insert(name.clone(), t.clone());

                vec![IR::StoreVar(name.clone(), e)]
            }

            //
            // return expr;
            //
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

            //
            // expr;
            //
            Stmt::ExprStmt(expr) => {
                let e = self.analyze_expr(expr, scope);

                // 임시 값 저장 (필요없지만 일관성을 위해 IR 생성)
                vec![IR::StoreVar("_expr_tmp".to_string(), e)]
            }

            //
            // if cond { then } else { else }
            //
            Stmt::If(cond, then_body, else_body) => {
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
                //
                // Built-in: 그냥 바로 허용
                //
                if self.builtins.contains(name) {
                    let ir_args = args
                        .iter()
                        .map(|a| self.analyze_expr(a, scope))
                        .collect();
                    return IRExpr::Call(name.clone(), ir_args);
                }

                //
                // 사용자 정의 함수
                //
                if !self.map.contains_key(name) {
                    panic!("Unknown function '{}'", name);
                }

                let func = self.map.get(name).unwrap();

                if func.params.len() != args.len() {
                    panic!(
                        "Argument count mismatch: expected {}, got {}",
                        func.params.len(),
                        args.len()
                    );
                }

                for (i, expr) in args.iter().enumerate() {
                    let arg_t = self.expr_type(expr, scope);
                    let param_t = &func.params[i].1;

                    if arg_t != *param_t {
                        panic!(
                            "Type mismatch for argument {} in {}: expected {:?}, got {:?}",
                            i, name, param_t, arg_t
                        );
                    }
                }

                let ir_args = args.iter().map(|a| self.analyze_expr(a, scope)).collect();
                IRExpr::Call(name.clone(), ir_args)
            }
        }
    }

    //
    // 타입 계산
    //
    fn expr_type(&self, expr: &Expr, scope: &HashMap<String, TypeName>) -> TypeName {
        match expr {
            Expr::Number(_) => TypeName::Int,

            Expr::StringLiteral(_) => TypeName::String,

            Expr::Var(name) => scope
                .get(name)
                .unwrap_or_else(|| panic!("Unknown variable '{}'", name))
                .clone(),

            Expr::Binary(a, op, b) => {
                let lt = self.expr_type(a, scope);
                let rt = self.expr_type(b, scope);

                // String + String → String
                if op == "+" && lt == TypeName::String && rt == TypeName::String {
                    return TypeName::String;
                }

                // 나머지는 전부 Int 연산
                if lt != TypeName::Int || rt != TypeName::Int {
                    panic!("Operator '{}' requires int operands", op);
                }

                TypeName::Int
            }

            Expr::Call(name, _) => {
                // built-in 함수는 타입이 없다 → print, println 은 아무거나 가능하게 하거나 Int 반환으로 고정 가능
                if self.builtins.contains(name) {
                    // println → Int 반환하도록 유지 (가장 편함)
                    return TypeName::Int;
                }

                let func = self
                    .map
                    .get(name)
                    .unwrap_or_else(|| panic!("Unknown function '{}'", name));
                func.ret_type.clone()
            }
        }
    }
}
