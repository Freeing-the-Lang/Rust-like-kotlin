use crate::semantic::*;
use std::fmt::Write;

pub struct Codegen;

// OS별 엔트리 심볼
#[cfg(target_os = "windows")]
const ENTRY: &str = "main";

#[cfg(not(target_os = "windows"))]
const ENTRY: &str = "_main";

impl Codegen {
    pub fn generate(&self, ir: &IRProgram) -> String {
        let mut out = String::new();

        // --------------------------------------------------------
        // ★ Windows/MSVC에서 반드시 필요한 코드 영역 선언
        // --------------------------------------------------------
        writeln!(&mut out, "section .text").unwrap();

        // Entry 선언
        writeln!(&mut out, "global {}", ENTRY).unwrap();

        // 각 함수 글로벌 선언
        for f in &ir.funcs {
            writeln!(&mut out, "global {}_func", f.name).unwrap();
            writeln!(&mut out, "global {}_func_end", f.name).unwrap();
        }

        // 실제 함수 코드 생성
        for f in &ir.funcs {
            self.gen_function(&mut out, f);
        }

        // ENTRY 래퍼
        if ir.funcs.iter().any(|f| f.name == "main") {
            writeln!(&mut out, "{}:", ENTRY).unwrap();
            writeln!(&mut out, "    call main_func").unwrap();
            writeln!(&mut out, "    ret").unwrap();
        }

        out
    }

    fn gen_function(&self, out: &mut String, f: &IRFunction) {
        let func_label = format!("{}_func", f.name);
        let end_label = format!("{}_func_end", f.name);

        // 함수 시작 라벨
        writeln!(out, "{}:", func_label).unwrap();

        // 함수 본문
        for stmt in &f.body {
            self.gen_stmt(out, stmt);
        }

        // 함수 끝
        writeln!(out, "{}:", end_label).unwrap();
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

            IR::If(cond, then_body, else_body) => {
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
                    "/" => {
                        writeln!(out, "    xor rdx, rdx").unwrap();
                        writeln!(out, "    idiv rcx").unwrap();
                    }
                    _ => {}
                };
            }

            _ => {}
        }
    }
}
