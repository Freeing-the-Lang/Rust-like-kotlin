use crate::semantic::{IRProgram, IRFunction, IR, IRExpr};

pub struct Codegen;

impl Codegen {
    pub fn new() -> Self {
        Codegen {}
    }

    // ============================================================
    //  ENTRY POINT (OS별 자동 처리)
    // ============================================================
    pub fn generate(&self, program: &IRProgram) -> String {
        let mut out = String::new();

        out.push_str("; =========================================\n");
        out.push_str(";   SpongeLang → IR → NASM Codegen (2025)  \n");
        out.push_str("; =========================================\n\n");

        // macOS는 _main, 그 외 OS는 main
        out.push_str("%ifdef __MACOS__\n");
        out.push_str("global _main\n");
        out.push_str("_main:\n");
        out.push_str("%else\n");
        out.push_str("global main\n");
        out.push_str("main:\n");
        out.push_str("%endif\n\n");

        out.push_str("    push rbp\n");
        out.push_str("    mov rbp, rsp\n");
        out.push_str("    call main_func\n");
        out.push_str("    mov rsp, rbp\n");
        out.push_str("    pop rbp\n");
        out.push_str("    ret\n\n");

        // 각 IR function 출력
        for func in &program.funcs {
            out.push_str(&self.generate_function(func));
        }

        out
    }

    // ============================================================
    //   FUNCTION CODEGEN
    // ============================================================
    fn generate_function(&self, f: &IRFunction) -> String {
        let mut out = String::new();

        if f.name == "main" {
            out.push_str("global main_func\n");
            out.push_str("main_func:\n");
        } else {
            out.push_str(&format!("global {}\n", f.name));
            out.push_str(&format!("{}:\n", f.name));
        }

        out.push_str("    push rbp\n");
        out.push_str("    mov rbp, rsp\n");

        let mut locals = Locals::new();

        for ir in &f.body {
            self.generate_stmt(ir, &mut out, &mut locals);
        }

        out.push_str("    mov rsp, rbp\n");
        out.push_str("    pop rbp\n");
        out.push_str("    ret\n\n");

        out
    }

    // ============================================================
    //   IR Statement Codegen
    // ============================================================
    fn generate_stmt(&self, ir: &IR, out: &mut String, locals: &mut Locals) {
        match ir {
            IR::StoreVar(name, expr) => {
                let offset = locals.get_or_create(name);
                self.generate_expr(expr, out, locals);
                out.push_str(&format!("    mov [rbp{}], rax\n", offset));
            }

            IR::LoadVar(name) => {
                let offset = locals.get_or_create(name);
                out.push_str(&format!("    mov rax, [rbp{}]\n", offset));
            }

            IR::Return(expr) => {
                self.generate_expr(expr, out, locals);
                out.push_str("    jmp .func_exit\n");
            }

            IR::If(cond, then_body, else_body) => {
                let l_then = self.new_label("then");
                let l_else = self.new_label("else");
                let l_end = self.new_label("endif");

                self.generate_expr(cond, out, locals);

                out.push_str("    cmp rax, 0\n");
                out.push_str(&format!("    jne {}\n", l_then));
                out.push_str(&format!("    jmp {}\n", l_else));

                out.push_str(&format!("{}:\n", l_then));
                for s in then_body {
                    self.generate_stmt(s, out, locals);
                }
                out.push_str(&format!("    jmp {}\n", l_end));

                out.push_str(&format!("{}:\n", l_else));
                for s in else_body {
                    self.generate_stmt(s, out, locals);
                }

                out.push_str(&format!("{}:\n", l_end));
            }

            IR::CallFunc(name, args) => {
                // push args (reverse order)
                for arg in args.iter().rev() {
                    self.generate_expr(arg, out, locals);
                    out.push_str("    push rax\n");
                }
                out.push_str(&format!("    call {}\n", name));
                out.push_str(&format!("    add rsp, {}\n", args.len() * 8));
            }

            IR::LiteralInt(_) | IR::LiteralString(_) | IR::BinaryOp(_,_,_) => {
                panic!("IRExpr should not appear directly in IR stmt level");
            }
        }
    }

    // ============================================================
    //   IR Expression Codegen
    // ============================================================
    fn generate_expr(&self, expr: &IRExpr, out: &mut String, locals: &Locals) {
        match expr {
            IRExpr::Int(n) => {
                out.push_str(&format!("    mov rax, {}\n", n));
            }

            IRExpr::Str(_) => {
                out.push_str("    mov rax, 0 ; string literal NYI\n");
            }

            IRExpr::Var(name) => {
                let offset = locals.get(name);
                out.push_str(&format!("    mov rax, [rbp{}]\n", offset));
            }

            IRExpr::Binary(a, op, b) => {
                self.generate_expr(a, out, locals);
                out.push_str("    push rax\n");
                self.generate_expr(b, out, locals);
                out.push_str("    mov rbx, rax\n");
                out.push_str("    pop rax\n");

                match op.as_str() {
                    "+" => out.push_str("    add rax, rbx\n"),
                    "-" => out.push_str("    sub rax, rbx\n"),
                    "*" => out.push_str("    imul rax, rbx\n"),
                    "/" => out.push_str("    cqo\n    idiv rbx\n"),
                    "==" => out.push_str("    cmp rax, rbx\n    sete al\n    movzx rax, al\n"),
                    "!=" => out.push_str("    cmp rax, rbx\n    setne al\n    movzx rax, al\n"),
                    ">"  => out.push_str("    cmp rax, rbx\n    setg al\n    movzx rax, al\n"),
                    "<"  => out.push_str("    cmp rax, rbx\n    setl al\n    movzx rax, al\n"),
                    _ => panic!("Unknown binary operator {}", op),
                }
            }

            IRExpr::Call(name, args) => {
                for arg in args.iter().rev() {
                    self.generate_expr(arg, out, locals);
                    out.push_str("    push rax\n");
                }
                out.push_str(&format!("    call {}\n", name));
                out.push_str(&format!("    add rsp, {}\n", args.len() * 8));
            }
        }
    }

    // ============================================================
    //  Unique label generator
    // ============================================================
    fn new_label(&self, prefix: &str) -> String {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CNT: AtomicUsize = AtomicUsize::new(0);
        let id = CNT.fetch_add(1, Ordering::SeqCst);
        format!(".{}_{}", prefix, id)
    }
}

// ============================================================
//  Local Variable Stack Layout Handler
// ============================================================
struct Locals {
    offset: i32,
    map: std::collections::HashMap<String, i32>,
}

impl Locals {
    fn new() -> Self {
        Self {
            offset: 0,
            map: std::collections::HashMap::new(),
        }
    }

    fn get_or_create(&mut self, name: &str) -> i32 {
        if let Some(v) = self.map.get(name) {
            return *v;
        }
        self.offset -= 8;
        self.map.insert(name.to_string(), self.offset);
        self.offset
    }

    fn get(&self, name: &str) -> i32 {
        *self.map.get(name).expect(&format!("Unknown local var {}", name))
    }
}
