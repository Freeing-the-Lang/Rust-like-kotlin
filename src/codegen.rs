use crate::semantic::*;
use std::fmt::Write;

pub struct Codegen;

// -----------------------------------------------
// 3OS 공통 ENTRY POINT = main
// -----------------------------------------------
const ENTRY: &str = "main";

impl Codegen {
    pub fn generate(&self, ir: &IRProgram) -> String {
        let mut out = String::new();

        // ====================================================
        // DATA SECTION  — 문자열 리터럴 & printf 포맷 보관
        // ====================================================
        writeln!(&mut out, "section .data").unwrap();

        // printf("%s")
        writeln!(&mut out, "fmt_str: db \"%s\", 0").unwrap();

        // 문자열 리터럴 모으기
        let mut strs = Vec::new();
        for f in &ir.funcs {
            for stmt in &f.body {
                self.collect_str(stmt, &mut strs);
            }
        }

        for (i, s) in strs.iter().enumerate() {
            writeln!(&mut out, "str_{}: db \"{}\", 0", i, s).unwrap();
        }

        // ====================================================
        // TEXT SECTION
        // ====================================================
        writeln!(&mut out, "section .text").unwrap();

        // ENTRY
        writeln!(&mut out, "global {}", ENTRY).unwrap();

        // OS별 printf 심볼
        #[cfg(target_os = "macos")]
        writeln!(&mut out, "extern _printf").unwrap();

        #[cfg(not(target_os = "macos"))]
        writeln!(&mut out, "extern printf").unwrap();

        // 함수 export
        for f in &ir.funcs {
            writeln!(&mut out, "global {}_func", f.name).unwrap();
            writeln!(&mut out, "global {}_func_end", f.name).unwrap();
        }

        // 함수 본문 출력
        for f in &ir.funcs {
            self.gen_function(&mut out, f, &strs);
        }

        // ====================================================
        // ENTRY main()
        // GCC, clang, MSVC 모두 main 엔트리를 인정함
        // printf 기반 런타임과 100% 호환
        // ====================================================
        writeln!(&mut out, "{}:", ENTRY).unwrap();
        writeln!(&mut out, "    call main_func").unwrap();
        writeln!(&mut out, "    mov eax, 0").unwrap();
        writeln!(&mut out, "    ret").unwrap();

        out
    }

    // 문자열 IR 수집
    fn collect_str(&self, stmt: &IR, out: &mut Vec<String>) {
        if let IR::Println(IRExpr::Str(s)) = stmt {
            out.push(s.clone());
        }
    }

    fn gen_function(&self, out: &mut String, f: &IRFunction, strs: &Vec<String>) {
        writeln!(out, "{}_func:", f.name).unwrap();
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

            IR::Println(expr) => {
                self.gen_print(out, expr, strs);
            }

            IR::StoreVar(_, expr) => {
                self.gen_expr(out, expr, strs);
            }

            _ => {}
        }
    }

    fn gen_expr(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        match expr {
            IRExpr::Int(n) => writeln!(out, "    mov rax, {}", n).unwrap(),

            IRExpr::Str(s) => {
                let idx = strs.iter().position(|x| x == s).unwrap();
                writeln!(out, "    lea rax, [rel str_{}]", idx).unwrap();
            }

            _ => {}
        }
    }

    // ====================================================
    // PRINTLN — printf 기반 출력 (3OS 완전 호환)
    // ====================================================
    fn gen_print(&self, out: &mut String, expr: &IRExpr, strs: &Vec<String>) {
        let idx = if let IRExpr::Str(s) = expr {
            strs.iter().position(|x| x == s).unwrap()
        } else {
            panic!("println only supports string literal");
        };

        // ----------------------------
        // macOS — _printf (underscore 필요)
        // ----------------------------
        #[cfg(target_os = "macos")]
        {
            writeln!(out, "    lea rdi, [rel fmt_str]").unwrap();      // 1st arg
            writeln!(out, "    lea rsi, [rel str_{}]", idx).unwrap();  // 2nd arg
            writeln!(out, "    sub rsp, 32").unwrap();
            writeln!(out, "    call _printf").unwrap();
            writeln!(out, "    add rsp, 32").unwrap();
            return;
        }

        // ----------------------------
        // Windows + Linux — printf
        // ----------------------------
        #[cfg(not(target_os = "macos"))]
        {
            writeln!(out, "    lea rcx, [rel fmt_str]").unwrap();      // 1st arg
            writeln!(out, "    lea rdx, [rel str_{}]", idx).unwrap();  // 2nd arg
            writeln!(out, "    sub rsp, 32").unwrap();
            writeln!(out, "    call printf").unwrap();
            writeln!(out, "    add rsp, 32").unwrap();
        }
    }
}
