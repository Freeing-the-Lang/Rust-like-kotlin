use crate::semantic::*;
use std::fmt::Write;

pub struct Codegen;

// OS별 ENTRY
#[cfg(target_os = "windows")]
const ENTRY: &str = "main";

#[cfg(not(target_os = "windows"))]
const ENTRY: &str = "_main";

impl Codegen {
    pub fn generate(&self, ir: &IRProgram) -> String {
        let mut out = String::new();

        // ------------------------------------
        // DATA 영역 (문자열 리터럴 저장)
        // ------------------------------------
        writeln!(&mut out, "section .data").unwrap();

        // 문자열 리터럴 테이블
        let mut str_literals = Vec::new();
        for func in &ir.funcs {
            for stmt in &func.body {
                Self::collect_str(stmt, &mut str_literals);
            }
        }

        for (i, s) in str_literals.iter().enumerate() {
            writeln!(&mut out, "str_{}: db \"{}\", 0", i, s).unwrap();
            writeln!(&mut out, "str_{}_len: equ $ - str_{}", i, i).unwrap();
        }

        // ------------------------------------
        // TEXT 영역
        // ------------------------------------
        writeln!(&mut out, "section .text").unwrap();

        // 모든 심볼 export
        writeln!(&mut out, "global {}", ENTRY).unwrap();
        for f in &ir.funcs {
            writeln!(&mut out, "global {}_func", f.name).unwrap();
            writeln!(&mut out, "global {}_func_end", f.name).unwrap();
        }

        // 함수 본문 생성
        for f in &ir.funcs {
            self.gen_function(&mut out, f, &str_literals);
        }

        // ENTRY wrapper
        if ir.funcs.iter().any(|f| f.name == "main") {
            writeln!(&mut out, "{}:", ENTRY).unwrap();
            writeln!(&mut out, "    call main_func").unwrap();
            writeln!(&mut out, "    ret").unwrap();
        }

        out
    }

    // 문자열 리터럴 재귀 수집
    fn collect_str(stmt: &IR, out: &mut Vec<String>) {
        match stmt {
            IR::Println(expr) => {
                if let IRExpr::Str(s) = expr {
                    out.push(s.clone());
                }
            }
            _ => {}
        }
    }

    fn gen_function(&self, out: &mut String, f: &IRFunction, strs: &Vec<String>) {
        let label = format!("{}_func", f.name);

        writeln!(out, "{}:", label).unwrap();

        for stmt in &f.body {
            self.gen_stmt(out, stmt, strs);
        }

        writeln!(out, "{}_func_end:", f.name).unwrap();
        writeln!(out, "    ret").unwrap();
    }

    fn gen_stmt(&self, out: &mut String, stmt: &IR, strs: &Vec<String>) {
        match stmt {
            IR::Return(expr) => {
                self.gen_expr(out, expr, strs);
                writeln!(out, "    ret").unwrap();
            }

            IR::StoreVar(_, expr) => {
                self.gen_expr(out, expr, strs);
            }

            IR::Println(expr) => {
                self.gen_print(out, expr, strs);
            }

            _ => {}
        }
    }

    fn gen_expr(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        match expr {
            IRExpr::Int(n) => {
                writeln!(out, "    mov rax, {}", n).unwrap();
            }

            IRExpr::Str(s) => {
                // 문자열 리터럴 위치 찾기
                let index = strs.iter().position(|x| x == s).unwrap();
                writeln!(out, "    lea rax, [rel str_{}]", index).unwrap();
            }

            _ => {}
        }
    }

    // --------------------------------------------------------
    // ★ println 구현 (3 OS 공통)
    // --------------------------------------------------------
    fn gen_print(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        // 문자열 주소 가져오기 → RDI (Linux/Mac), RDX (Windows)
        if let IRExpr::Str(s) = expr {
            let idx = strs.iter().position(|x| x == s).unwrap();

            // OS 별 출력 방식
            #[cfg(target_os = "windows")]
            {
                writeln!(out, "    ; println (Windows)").unwrap();
                writeln!(out, "    sub rsp, 32").unwrap();
                writeln!(out, "    mov rcx, -11").unwrap();        // STD_OUTPUT_HANDLE
                writeln!(out, "    call GetStdHandle").unwrap();
                writeln!(out, "    mov rcx, rax").unwrap();        // handle
                writeln!(out, "    lea rdx, [rel str_{}]", idx).unwrap();
                writeln!(out, "    mov r8, str_{}_len", idx).unwrap();
                writeln!(out, "    xor r9d, r9d").unwrap();
                writeln!(out, "    call WriteConsoleA").unwrap();
                writeln!(out, "    add rsp, 32").unwrap();
            }

            #[cfg(target_os = "linux")]
            {
                writeln!(out, "    ; println (Linux)").unwrap();
                writeln!(out, "    mov rax, 1").unwrap();                // write
                writeln!(out, "    mov rdi, 1").unwrap();                // stdout
                writeln!(out, "    lea rsi, [rel str_{}]", idx).unwrap();
                writeln!(out, "    mov rdx, str_{}_len", idx).unwrap();
                writeln!(out, "    syscall").unwrap();
            }

            #[cfg(target_os = "macos")]
            {
                writeln!(out, "    ; println (macOS)").unwrap();
                writeln!(out, "    mov rax, 0x2000004").unwrap();         // write
                writeln!(out, "    mov rdi, 1").unwrap();                // stdout
                writeln!(out, "    lea rsi, [rel str_{}]", idx).unwrap();
                writeln!(out, "    mov rdx, str_{}_len", idx).unwrap();
                writeln!(out, "    syscall").unwrap();
            }
        }
    }
            }
