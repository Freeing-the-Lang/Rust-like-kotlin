use crate::parser::{Expr, Function, Program, Stmt};

pub struct Codegen;

impl Codegen {
    pub fn new() -> Self {
        Codegen {}
    }

    pub fn generate(&self, program: &Program) -> String {
        let mut out = String::new();

        out.push_str("; ============================\n");
        out.push_str(";   SpongeLang NASM Codegen   \n");
        out.push_str("; ============================\n\n");

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

        for func in &program.funcs {
            out.push_str(&self.generate_function(func));
        }

        out
    }

    fn generate_function(&self, func: &Function) -> String {
        let mut out = String::new();

        if func.name == "main" {
            out.push_str("global main_func\n");
            out.push_str("main_func:\n");
        } else {
            out.push_str(&format!("global {}\n", func.name));
            out.push_str(&format!("{}:\n", func.name));
        }

        out.push_str("    push rbp\n");
        out.push_str("    mov rbp, rsp\n");

        let mut local_offsets = Vec::new();
        let mut offset = 0;

        for stmt in &func.body {
            self.generate_stmt(stmt, &mut out, &mut offset, &mut local_offsets);
        }

        out.push_str("    mov rsp, rbp\n");
        out.push_str("    pop rbp\n");
        out.push_str("    ret\n\n");

        out
    }

    fn generate_stmt(
        &self,
        stmt: &Stmt,
        out: &mut String,
        offset: &mut i32,
        locals: &mut Vec<(String, i32)>,
    ) {
        match stmt {
            Stmt::Let(name, _, expr) => {
                *offset -= 8;
                locals.push((name.clone(), *offset));

                self.generate_expr(expr, out, locals);
                out.push_str(&format!("    mov [rbp{}], rax\n", *offset));
            }

            Stmt::ExprStmt(expr) => {
                self.generate_expr(expr, out, locals);
            }

            Stmt::Return(expr) => {
                self.generate_expr(expr, out, locals);
                out.push_str("    jmp .func_exit\n");
            }

            Stmt::If(cond, then_body, else_body) => {
                let then_label = self.new_label("then");
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");

                self.generate_expr(cond, out, locals);
                out.push_str(&format!("    cmp rax, 0\n"));
                out.push_str(&format!("    jne {}\n", then_label));
                out.push_str(&format!("    jmp {}\n", else_label));

                out.push_str(&format!("{}:\n", then_label));
                for s in then_body {
                    self.generate_stmt(s, out, offset, locals);
                }
                out.push_str(&format!("    jmp {}\n", end_label));

                out.push_str(&format!("{}:\n", else_label));
                for s in else_body {
                    self.generate_stmt(s, out, offset, locals);
                }

                out.push_str(&format!("{}:\n", end_label));
            }
        }
    }

    fn generate_expr(&self, expr: &Expr, out: &mut String, locals: &Vec<(String, i32)>) {
        match expr {
            Expr::Number(n) => {
                out.push_str(&format!("    mov rax, {}\n", n));
            }

            Expr::StringLiteral(_s) => {
                out.push_str("    mov rax, 0 ; string literal not implemented\n");
            }

            Expr::Var(name) => {
                for (n, off) in locals {
                    if n == name {
                        out.push_str(&format!("    mov rax, [rbp{}]\n", off));
                        return;
                    }
                }
                panic!("Undefined variable: {}", name);
            }

            Expr::Binary(lhs, op, rhs) => {
                self.generate_expr(lhs, out, locals);
                out.push_str("    push rax\n");

                self.generate_expr(rhs, out, locals);
                out.push_str("    mov rbx, rax\n");

                out.push_str("    pop rax\n");

                match op.as_str() {
                    "+" => out.push_str("    add rax, rbx\n"),
                    "-" => out.push_str("    sub rax, rbx\n"),
                    "*" => out.push_str("    imul rax, rbx\n"),
                    "/" => out.push_str("    cqo\n    idiv rbx\n"),
                    ">" => out.push_str("    cmp rax, rbx\n    setg al\n    movzx rax, al\n"),
                    "<" => out.push_str("    cmp rax, rbx\n    setl al\n    movzx rax, al\n"),
                    "==" => out.push_str("    cmp rax, rbx\n    sete al\n    movzx rax, al\n"),
                    "!=" => out.push_str("    cmp rax, rbx\n    setne al\n    movzx rax, al\n"),
                    _ => panic!("Unknown binary op {}", op),
                }
            }

            Expr::Call(name, args) => {
                for (i, arg) in args.iter().enumerate().rev() {
                    self.generate_expr(arg, out, locals);
                    out.push_str("    push rax\n");
                }

                out.push_str(&format!("    call {}\n", name));

                out.push_str(&format!("    add rsp, {}\n", args.len() * 8));
            }
        }
    }

    fn new_label(&self, base: &str) -> String {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!(".{}_{}", base, id)
    }
}
