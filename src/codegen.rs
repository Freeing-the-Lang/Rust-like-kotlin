use crate::semantic::*;
use std::fmt::Write;

pub struct Codegen;

// OS별 엔트리 심볼 자동 선택
#[cfg(target_os = "windows")]
const ENTRY: &str = "main";

#[cfg(not(target_os = "windows"))]
const ENTRY: &str = "_main";

impl Codegen {
    pub fn generate(&self, ir: &IRProgram) -> String {
        let mut out = String::new();

        // global 선언
        writeln!(&mut out, "global {}", ENTRY).unwrap();

        // 각 함수 생성
        for f in &ir.funcs {
            self.gen_function(&mut out, f);
        }

        // main 함수가 존재하면 엔트리 생성
        if ir.funcs.iter().any(|f| f.name == "main") {
            writeln!(&mut out, "{}:", ENTRY).unwrap();
            writeln!(&mut out, "    call main_func").unwrap();
            writeln!(&mut out, "    ret").unwrap();
        }

        out
    }

    fn gen_function(&self, out: &mut String, f: &IRFunction) {
        let name = format!("{}_func", f.name);

        writeln!(out, "{}:", name).unwrap();

        for stmt in &f.body {
            self.gen_stmt(out, stmt);
        }

        writeln!(out, "{}_end:", name).unwrap();
        writeln!(out, "    ret").unwrap();
    }

    fn gen_stmt(&self, out: &mut String, stmt: &IR) {
        match stmt {
            IR::Return(expr) => {
                self.gen_expr(out, expr);
                writeln!(out, "    ret").unwrap();
            }

            IR::StoreVar(name, expr) => {
                self.gen_expr(out, expr);
                writeln!(out, "    ; store var {}", name).unwrap();
            }

            IR::LoadVar(name) => {
                writeln!(out, "    ; load var {}", name).unwrap();
            }

            IR::If(cond, then_body, else_body) => {
                writeln!(out, "    ; if start").unwrap();
                self.gen_expr(out, cond);

                writeln!(out, "    cmp rax, 0").unwrap();
                writeln!(out, "    je .else_block").unwrap();

                for s in then_body {
                    self.gen_stmt(out, s);
                }

                writeln!(out, "    jmp .end_if").unwrap();
                writeln!(out, ".else_block:").unwrap();

                for s in else_body {
                    self.gen_stmt(out, s);
                }

                writeln!(out, ".end_if:").unwrap();
            }

            _ => {}
        }
    }

    fn gen_expr(&self, out: &mut String, expr: &IRExpr) {
        match expr {
            IRExpr::Int(n) => {
                writeln!(out, "    mov rax, {}", n).unwrap();
            }

            IRExpr::Binary(a, op, b) => {
                self.gen_expr(out, a);
                writeln!(out, "    push rax").unwrap();
                self.gen_expr(out, b);
                writeln!(out, "    pop rcx").unwrap();

                match op.as_str() {
                    "+" => writeln!(out, "    add rax, rcx").unwrap(),
                    "-" => writeln!(out, "    sub rax, rcx").unwrap(),
                    "*" => writeln!(out, "    imul rax, rcx").unwrap(),
                    "/" => writeln!(out, "    xor rdx, rdx\n    idiv rcx").unwrap(),
                    _ => {}
                };
            }

            _ => {}
        }
    }
}
